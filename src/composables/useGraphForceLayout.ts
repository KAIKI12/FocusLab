/**
 * useGraphForceLayout · d3-force 力布局封装
 *
 * 设计要点:
 * 1. 仿真实例与 d3 内部数组用 markRaw 包裹,避免 Vue proxy 代理 d3 内部数据
 * 2. positions 用 shallowRef<Map>,按 rAF 节流写入,降低 reactive 开销
 * 3. 增量更新: 节点新增时保留旧节点的 x/y/vx/vy,新节点沿圆周分布初始位置
 * 4. alpha < alphaMin 时由 d3 自动 stop(),省电
 *
 * 不在本期实现: Web Worker 化(>100 节点时考虑)
 */

import {
  markRaw,
  onScopeDispose,
  shallowRef,
  toValue,
  triggerRef,
  watch,
  type MaybeRefOrGetter,
  type Ref,
} from "vue";
import {
  forceCenter,
  forceCollide,
  forceLink,
  forceManyBody,
  forceSimulation,
  type ForceLink,
  type Simulation,
  type SimulationLinkDatum,
  type SimulationNodeDatum,
} from "d3-force";

interface ForceNodeInput {
  id: string;
}

interface ForceEdgeInput {
  source: string;
  target: string;
}

interface InternalNode extends SimulationNodeDatum {
  id: string;
}

type InternalLink = SimulationLinkDatum<InternalNode>;

export interface UseGraphForceLayoutOptions {
  width: number;
  height: number;
  /** link 期望距离, 默认 110 */
  linkDistance?: number;
  /** 节点排斥力, 默认 -260 */
  charge?: number;
  /** 节点碰撞半径, 默认 34 */
  collideRadius?: number;
  /** alpha 衰减速率, 默认 0.045 (越大越快稳定) */
  alphaDecay?: number;
  /** 中心力强度, 默认 0.05 */
  centerStrength?: number;
}

export interface GraphForceLayoutHandle {
  /** 节点位置 Map(id → {x, y}) , 每帧 rAF 内更新 */
  positions: Readonly<Ref<Map<string, { x: number; y: number }>>>;
  /** 主动重启仿真,可指定 alpha (默认 0.5) */
  restart: (alpha?: number) => void;
  /** 拖拽:固定节点位置 */
  pin: (id: string, x: number, y: number) => void;
  /** 拖拽结束:解除固定 */
  unpin: (id: string) => void;
  /** 容器尺寸变化时调用 */
  resize: (width: number, height: number) => void;
  /** 销毁 */
  dispose: () => void;
}

export function useGraphForceLayout(
  nodesSource: MaybeRefOrGetter<ForceNodeInput[]>,
  edgesSource: MaybeRefOrGetter<ForceEdgeInput[]>,
  opts: UseGraphForceLayoutOptions,
): GraphForceLayoutHandle {
  const positions = shallowRef<Map<string, { x: number; y: number }>>(new Map());

  // d3 内部数组用 markRaw,避免 Vue 代理 d3 内部 mutation
  const internalNodes: InternalNode[] = markRaw([]);
  const internalLinks: InternalLink[] = markRaw([]);
  const nodeById = new Map<string, InternalNode>();

  let rafScheduled = false;
  let containerWidth = opts.width;
  let containerHeight = opts.height;

  function flushPositions() {
    rafScheduled = false;
    const next = new Map<string, { x: number; y: number }>();
    for (const node of internalNodes) {
      if (typeof node.x === "number" && typeof node.y === "number") {
        next.set(node.id, { x: node.x, y: node.y });
      }
    }
    positions.value = next;
    triggerRef(positions);
  }

  function scheduleFlush() {
    if (rafScheduled) return;
    rafScheduled = true;
    requestAnimationFrame(flushPositions);
  }

  // 显式持有 link force 引用,避免 simulation.force<T>(name) 的类型麻烦
  const linkForce = forceLink<InternalNode, InternalLink>(internalLinks)
    .id((d) => d.id)
    .distance(opts.linkDistance ?? 110)
    .strength(0.4);

  const centerForce = forceCenter<InternalNode>(
    containerWidth / 2,
    containerHeight / 2,
  ).strength(opts.centerStrength ?? 0.05);

  const simulation: Simulation<InternalNode, InternalLink> = markRaw(
    forceSimulation<InternalNode, InternalLink>(internalNodes)
      .force("link", linkForce as ForceLink<InternalNode, InternalLink>)
      .force(
        "charge",
        forceManyBody<InternalNode>().strength(opts.charge ?? -260).distanceMax(420),
      )
      .force("center", centerForce)
      .force(
        "collide",
        forceCollide<InternalNode>().radius(opts.collideRadius ?? 34).iterations(2),
      )
      .alphaDecay(opts.alphaDecay ?? 0.045)
      .alphaMin(0.002)
      .velocityDecay(0.35)
      .on("tick", scheduleFlush)
      .on("end", flushPositions),
  );

  function syncNodesAndLinks() {
    const nextNodes = toValue(nodesSource);
    const nextEdges = toValue(edgesSource);
    const seen = new Set<string>();

    // 用现有节点几何中心作为新节点种子,否则用容器中心
    let cx = containerWidth / 2;
    let cy = containerHeight / 2;
    if (internalNodes.length > 0) {
      let sx = 0;
      let sy = 0;
      let n = 0;
      for (const node of internalNodes) {
        if (typeof node.x === "number" && typeof node.y === "number") {
          sx += node.x;
          sy += node.y;
          n += 1;
        }
      }
      if (n > 0) {
        cx = sx / n;
        cy = sy / n;
      }
    }

    // 1) 节点 diff: 旧节点保留 x/y/vx/vy, 新节点圆周分布初始位置
    const next: InternalNode[] = [];
    let newCount = 0;
    const totalCount = Math.max(nextNodes.length, 1);
    // 初始半径自适应容器尺寸:小容器 ~80px,大容器最多到 min(w,h)*0.35
    // 太小则节点全挤中心,d3 simulation 散开慢,用户感觉"全在中间"
    const initRadius = Math.max(
      80,
      Math.min(containerWidth, containerHeight) * 0.35,
    );
    for (let i = 0; i < nextNodes.length; i += 1) {
      const input = nextNodes[i];
      seen.add(input.id);
      const existing = nodeById.get(input.id);
      if (existing) {
        next.push(existing);
      } else {
        const angle = (Math.PI * 2 * i) / totalCount;
        const node: InternalNode = {
          id: input.id,
          x: cx + Math.cos(angle) * initRadius,
          y: cy + Math.sin(angle) * initRadius,
        };
        nodeById.set(input.id, node);
        next.push(node);
        newCount += 1;
      }
    }

    // 2) 移除已删节点
    let removedCount = 0;
    for (const id of Array.from(nodeById.keys())) {
      if (!seen.has(id)) {
        nodeById.delete(id);
        removedCount += 1;
      }
    }

    // 3) 替换数组内容 (d3 持引用,需 in-place 替换)
    internalNodes.splice(0, internalNodes.length, ...next);

    // 4) 重建 links (id-based;forceLink.id() 会自动解析)
    internalLinks.splice(
      0,
      internalLinks.length,
      ...nextEdges.map(
        (edge): InternalLink => ({ source: edge.source, target: edge.target }),
      ),
    );

    // 5) 通知 d3
    simulation.nodes(internalNodes);
    linkForce.links(internalLinks);

    // 6) 集合变化才重启,避免无意义弹跳
    if (newCount > 0 || removedCount > 0) {
      // 新节点引入或节点被删 → alpha 拉高让节点足够散开
      simulation.alpha(0.8).restart();
    } else if (internalLinks.length > 0) {
      simulation.alpha(0.15).restart();
    }
  }

  // 初次同步
  syncNodesAndLinks();

  // 节点/边变化时增量同步:用 id-join 字符串做 cheap diff
  const stopWatchNodes = watch(
    () => toValue(nodesSource).map((n) => n.id).join("|"),
    () => syncNodesAndLinks(),
  );
  const stopWatchEdges = watch(
    () => toValue(edgesSource).map((e) => `${e.source}->${e.target}`).join("|"),
    () => syncNodesAndLinks(),
  );

  function restart(alpha = 0.5) {
    simulation.alpha(alpha).restart();
  }

  function pin(id: string, x: number, y: number) {
    const node = nodeById.get(id);
    if (!node) return;
    node.fx = x;
    node.fy = y;
    simulation.alphaTarget(0.1).restart();
  }

  function unpin(id: string) {
    const node = nodeById.get(id);
    if (!node) return;
    node.fx = null;
    node.fy = null;
    simulation.alphaTarget(0);
  }

  function resize(width: number, height: number) {
    containerWidth = width;
    containerHeight = height;
    centerForce.x(width / 2).y(height / 2);
    simulation.alpha(0.2).restart();
  }

  function dispose() {
    stopWatchNodes();
    stopWatchEdges();
    simulation.on("tick", null);
    simulation.on("end", null);
    simulation.stop();
    internalNodes.splice(0);
    internalLinks.splice(0);
    nodeById.clear();
  }

  onScopeDispose(dispose);

  return { positions, restart, pin, unpin, resize, dispose };
}

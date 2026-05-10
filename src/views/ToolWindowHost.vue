<script setup lang="ts">
import { getCurrentWebviewWindow } from "@tauri-apps/api/webviewWindow";

import CommandPalette from "@/components/common/CommandPalette.vue";
import QuickAddWindowView from "@/views/QuickAddWindowView.vue";
import QuickNoteWindowView from "@/views/QuickNoteWindowView.vue";

const label = getCurrentWebviewWindow().label;
</script>

<template>
  <QuickAddWindowView v-if="label === 'quick-add'" />
  <QuickNoteWindowView v-else-if="label === 'quick-note'" />
  <CommandPalette v-else standalone />
</template>

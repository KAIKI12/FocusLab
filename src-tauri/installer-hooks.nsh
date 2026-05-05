!macro NSIS_HOOK_PREINSTALL
  IfFileExists "$APPDATA\com.focuslab.app\focuslab.db" 0 done
    MessageBox MB_YESNO|MB_ICONQUESTION|MB_DEFBUTTON2 \
      "检测到已存在的 FocusLab 本地数据。$\r$\n$\r$\n是否删除旧数据后继续安装？$\r$\n选择“否”会保留已有数据。" \
      /SD IDNO IDNO done
    RMDir /r "$APPDATA\com.focuslab.app"
  done:
!macroend

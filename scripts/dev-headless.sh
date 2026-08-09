#!/usr/bin/env bash
# Запуск лаунчера в headless-окружении (без реального монитора).
# Поднимает виртуальный X-сервер (Xvfb) и запускает `tauri dev` с бэкендом X11.
set -euo pipefail

DISP=${DISPLAY_HEADLESS:-:99}

if ! xdpyinfo -display "$DISP" >/dev/null 2>&1; then
  echo "Запускаю Xvfb на $DISP..."
  Xvfb "$DISP" -screen 0 1280x800x24 >/tmp/xvfb.log 2>&1 &
  sleep 2
fi

export DISPLAY="$DISP"
export GDK_BACKEND=x11
# Софтверный рендеринг, чтобы обойти отсутствие DRI3 в Xvfb.
export LIBGL_ALWAYS_SOFTWARE=1
# Обход пустого/белого окна в WebKitGTK без GPU-композитинга.
export WEBKIT_DISABLE_COMPOSITING_MODE=1
export WEBKIT_DISABLE_DMABUF_RENDERER=1

echo "Запуск лаунчера на $DISP (GDK_BACKEND=x11)"
npm run tauri dev

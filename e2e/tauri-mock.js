/**
 * Tauri IPC / Event Mock Runtime for Playwright E2E
 *
 * 通过 page.addInitScript 注入到浏览器页面，替代真实的 Tauri 后端。
 * 覆盖 window.__TAURI_INTERNALS__，支持 invoke、transformCallback、runCallback。
 * 同时内置 plugin:event|listen / emit / unlisten 的 mock 实现。
 */
(() => {
  if (window.__TAURI_INTERNALS__?.invoke) return;

  const callbacks = new Map();
  let callbackIdSeq = 0;

  /** 事件监听器：event -> handlerId[] */
  const eventListeners = new Map();

  /** 用户命令 mock handler */
  let commandHandler = () => {
    throw new Error("TAURI_MOCK: commandHandler not set");
  };

  window.__TAURI_INTERNALS__ = {
    invoke: (cmd, args = {}) => {
      // --- 内置事件系统 ---
      if (cmd === "plugin:event|listen") {
        const list = eventListeners.get(args.event) || [];
        list.push(args.handler);
        eventListeners.set(args.event, list);
        return Promise.resolve(args.handler);
      }
      if (cmd === "plugin:event|emit") {
        const list = eventListeners.get(args.event) || [];
        list.forEach((id) => {
          const cb = callbacks.get(id);
          if (cb) {
            cb({ event: args.event, payload: args.payload, id: 0 });
          }
        });
        return Promise.resolve(null);
      }
      if (cmd === "plugin:event|unlisten") {
        const list = eventListeners.get(args.event);
        if (list) {
          const idx = list.indexOf(args.eventId);
          if (idx > -1) list.splice(idx, 1);
        }
        return Promise.resolve(null);
      }

      // --- 用户命令 ---
      try {
        const result = commandHandler(cmd, args);
        return Promise.resolve(result);
      } catch (e) {
        return Promise.reject(e);
      }
    },

    transformCallback: (cb, once = false) => {
      const id = callbackIdSeq++;
      const wrapped = (payload) => {
        if (once) callbacks.delete(id);
        cb(payload);
      };
      callbacks.set(id, wrapped);
      return id;
    },

    runCallback: (id, payload) => {
      const cb = callbacks.get(id);
      if (cb) cb(payload);
    },

    convertFileSrc: (path, protocol = "asset") => {
      return `http://${protocol}.localhost/${encodeURIComponent(path)}`;
    },
  };

  window.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
    unregisterListener: () => {},
  };

  /** 测试侧调用：设置命令 mock handler */
  window.__setTauriMockHandler__ = (fn) => {
    commandHandler = fn;
  };

  /** 测试侧调用：直接触发事件（绕过 invoke 层，更高效） */
  window.__emitTauriEvent__ = (event, payload) => {
    const list = eventListeners.get(event) || [];
    list.forEach((id) => {
      const cb = callbacks.get(id);
      if (cb) cb({ event, payload, id: 0 });
    });
  };

  /** 测试侧调用：清空所有 mocks（页面刷新后不需要，因为每次 test 新 page） */
  window.__clearTauriMocks__ = () => {
    callbacks.clear();
    callbackIdSeq = 0;
    eventListeners.clear();
    commandHandler = () => {
      throw new Error("TAURI_MOCK: commandHandler not set");
    };
  };
})();

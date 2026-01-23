import init, { LogProcessor } from "/wasm/serial_monitor.js";

let processor, lastNotify = 0, pending = null, currentFile = null;

const notify = (count) => {
    if (Date.now() - lastNotify > 50) {
        lastNotify = Date.now();
        self.postMessage({ type: 'TOTAL_LINES', data: count });
        if (pending) clearTimeout(pending), pending = null;
    } else if (!pending) pending = setTimeout(() => notify(processor.get_line_count()), 50);
};

const getFiles = async (root) => {
    const files = [];
    for await (const [n, h] of root.entries()) if (n.startsWith('logs_') && n.endsWith('.txt')) files.push([n, h]);
    return files.sort((a, b) => parseInt(b[0].split('_')[1]) - parseInt(a[0].split('_')[1]));
};

// Retry wrapper for OPFS exclusive locking
const getLock = async (fileHandle) => {
    for (let i = 0; i < 20; i++) {
        try { return await fileHandle.createSyncAccessHandle(); }
        catch (e) {
            // Wait 100ms if locked
            if (e.name === 'NoModificationAllowedError' || e.name === 'InvalidStateError') {
                await new Promise(r => setTimeout(r, 100));
                continue;
            }
            throw e;
        }
    }
    throw new Error("Failed to acquire OPFS lock after retries");
};

const newSession = async (root, cleanup = false) => {
    if (cleanup && currentFile) try { await root.removeEntry(currentFile); } catch (e) { }
    currentFile = `logs_${Date.now()}.txt`;
    const h = await getLock(await root.getFileHandle(currentFile, { create: true }));
    processor.set_sync_handle(h);
    processor.clear();
    return h;
};

(async () => {
    try {
        await init();
        processor = new LogProcessor();
        const root = await navigator.storage.getDirectory();

        const files = await getFiles(root);
        if (files.length > 0) {
            currentFile = files[0][0];
            try {
                // Try to resume session
                const h = await getLock(files[0][1]);
                processor.set_sync_handle(h); // Rebuild indices
            } catch (e) {
                console.warn("Resume failed (lock?), starting new session", e);
                await newSession(root); // Fallback if lock fails completely
            }
            // Cleanup stale
            for (let i = 1; i < files.length; i++) try { await root.removeEntry(files[i][0]); } catch (e) { }
        } else {
            await newSession(root);
        }

        self.postMessage({ type: 'INITIALIZED' });
        // Only send total lines if we successfully processed existing logs
        if (processor.get_line_count() > 0) self.postMessage({ type: 'TOTAL_LINES', data: processor.get_line_count() });

        self.onmessage = async ({ data: { type, data } }) => {
            try {
                if (type === 'NEW_SESSION') { await newSession(root, true); self.postMessage({ type: 'TOTAL_LINES', data: 0 }); }
                else if (type === 'APPEND_CHUNK') { processor.append_chunk(data.chunk, data.is_hex); notify(processor.get_line_count()); }
                else if (type === 'REQUEST_WINDOW') self.postMessage({ type: 'LOG_WINDOW', data: { startLine: data.startLine, lines: processor.request_window(data.startLine, data.count) } });
                else if (type === 'SEARCH_LOGS') { processor.search_logs(data.query, data.match_case, data.use_regex, data.invert); notify(processor.get_line_count()); }
                else if (type === 'EXPORT_LOGS') { const s = processor.export_logs(!(data?.include_timestamp === false)); self.postMessage({ type: 'EXPORT_STREAM', stream: s }, [s]); }
                else if (type === 'CLEAR') { processor.clear(); self.postMessage({ type: 'TOTAL_LINES', data: 0 }); }
                else if (type === 'SET_LINE_ENDING') processor.set_line_ending(data);
            } catch (e) { console.error(e); }
        };
    } catch (e) { console.error("Worker Init Failed", e); }
})();

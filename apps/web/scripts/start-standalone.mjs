// The generated server defaults to all interfaces; this local application always binds loopback.
process.env.HOSTNAME = '127.0.0.1';
await import('../.next/standalone/apps/web/server.js');

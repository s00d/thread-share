#!/usr/bin/env node

/**
 * Simple TCP Socket Server for ThreadShare Rust Client Demo
 * No dependencies required - pure Node.js
 */

const net = require('net');

class SocketServer {
    constructor(port = 8080) {
        this.port = port;
        this.clients = new Map();
        this.messageCounter = 0;
        this.server = null;
        this.isRunning = false;
    }

    start() {
        this.server = net.createServer((socket) => {
            this.handleConnection(socket);
        });

        this.server.listen(this.port, () => {
            this.isRunning = true;
            console.log(`🚀 Socket Server started on port ${this.port}`);
            console.log(`📡 Waiting for connections...`);
            console.log(`💡 Connect with: telnet localhost ${this.port}`);
            console.log(`💡 Or use the Rust client example`);
        });

        this.server.on('error', (err) => {
            if (err.code === 'EADDRINUSE') {
                console.error(`❌ Port ${this.port} is already in use`);
                console.error(`💡 Try a different port or stop the existing server`);
            } else {
                console.error('❌ Server error:', err.message);
            }
        });

        // Graceful shutdown
        process.on('SIGINT', () => {
            console.log('\n🛑 Shutting down server...');
            this.stop();
            process.exit(0);
        });

        process.on('SIGTERM', () => {
            console.log('\n🛑 Shutting down server...');
            this.stop();
            process.exit(0);
        });
    }

    handleConnection(socket) {
        const clientId = this.generateClientId();
        const clientInfo = {
            id: clientId,
            address: socket.remoteAddress,
            port: socket.remotePort,
            connectedAt: new Date(),
            messagesReceived: 0,
            messagesSent: 0
        };

        this.clients.set(clientId, clientInfo);

        console.log(`\n🔌 New client connected: ${clientId}`);
        console.log(`📍 Address: ${clientInfo.address}:${clientInfo.port}`);
        console.log(`⏰ Connected at: ${clientInfo.connectedAt.toLocaleTimeString()}`);

        // Send welcome message
        const welcomeMsg = `Welcome to ThreadShare Socket Server! Client ID: ${clientId}`;
        socket.write(welcomeMsg + '\n');
        clientInfo.messagesSent++;

        // Handle incoming data
        socket.on('data', (data) => {
            const message = data.toString().trim();
            if (message) {
                this.handleMessage(socket, clientId, message);
            }
        });

        // Handle client disconnect
        socket.on('close', () => {
            this.handleDisconnect(clientId);
        });

        // Handle errors
        socket.on('error', (err) => {
            console.error(`❌ Client ${clientId} error:`, err.message);
            this.handleDisconnect(clientId);
        });
    }

    handleMessage(socket, clientId, message) {
        const client = this.clients.get(clientId);
        if (!client) return;

        client.messagesReceived++;
        this.messageCounter++;

        console.log(`📥 [${clientId}] Received: ${message}`);
        console.log(`📊 Total messages: ${this.messageCounter}`);

        // Process message and send response
        const response = this.processMessage(message, clientId);
        socket.write(response + '\n');
        client.messagesSent++;

        console.log(`📤 [${clientId}] Sent: ${response}`);
    }

    processMessage(message, clientId) {
        // Simple message processing logic
        if (message.toLowerCase().includes('hello')) {
            return `Hello back from server! Client ${clientId}`;
        } else if (message.toLowerCase().includes('ping')) {
            return `Pong! Server time: ${new Date().toLocaleTimeString()}`;
        } else if (message.toLowerCase().includes('status')) {
            const client = this.clients.get(clientId);
            return `Status: Connected for ${this.getUptime(client.connectedAt)}, Messages: ${client.messagesReceived}`;
        } else if (message.toLowerCase().includes('help')) {
            return `Available commands: hello, ping, status, help, quit`;
        } else if (message.toLowerCase().includes('quit')) {
            return `Goodbye! Disconnecting...`;
        } else {
            // Echo the message back with some processing
            const processedMessage = message
                .split('')
                .reverse()
                .join('')
                .toUpperCase();
            
            return `Echo: ${processedMessage} (Original: ${message})`;
        }
    }

    handleDisconnect(clientId) {
        const client = this.clients.get(clientId);
        if (client) {
            const uptime = this.getUptime(client.connectedAt);
            console.log(`\n🔌 Client ${clientId} disconnected`);
            console.log(`⏱️  Uptime: ${uptime}`);
            console.log(`📥 Messages received: ${client.messagesReceived}`);
            console.log(`📤 Messages sent: ${client.messagesSent}`);
            
            this.clients.delete(clientId);
        }

        console.log(`📊 Active clients: ${this.clients.size}`);
    }

    generateClientId() {
        return `client_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    }

    getUptime(connectedAt) {
        const now = new Date();
        const diff = now - connectedAt;
        const seconds = Math.floor(diff / 1000);
        const minutes = Math.floor(seconds / 60);
        const hours = Math.floor(minutes / 60);

        if (hours > 0) {
            return `${hours}h ${minutes % 60}m ${seconds % 60}s`;
        } else if (minutes > 0) {
            return `${minutes}m ${seconds % 60}s`;
        } else {
            return `${seconds}s`;
        }
    }

    getStats() {
        const totalMessages = Array.from(this.clients.values())
            .reduce((sum, client) => sum + client.messagesReceived, 0);
        
        const totalSent = Array.from(this.clients.values())
            .reduce((sum, client) => sum + client.messagesSent, 0);

        return {
            activeClients: this.clients.size,
            totalMessages,
            totalSent,
            serverUptime: this.isRunning ? 'Running' : 'Stopped'
        };
    }

    stop() {
        if (this.server) {
            this.server.close(() => {
                console.log('✅ Server stopped');
            });
            
            // Close all client connections
            for (const [clientId, client] of this.clients) {
                console.log(`🔌 Closing connection to ${clientId}`);
            }
            this.clients.clear();
            
            this.isRunning = false;
        }
    }
}

// Start the server
const server = new SocketServer(8080);

// Display server info
console.log('🎯 ThreadShare Socket Server Demo');
console.log('================================');
console.log('This server demonstrates:');
console.log('• TCP socket handling');
console.log('• Client connection management');
console.log('• Message processing and responses');
console.log('• Statistics tracking');
console.log('• Graceful shutdown');
console.log('');

server.start();

// Display stats every 10 seconds
setInterval(() => {
    const stats = server.getStats();
    if (stats.activeClients > 0) {
        console.log(`\n📊 Server Stats: ${stats.activeClients} clients, ${stats.totalMessages} messages received, ${stats.totalSent} messages sent`);
    }
}, 10000);

// Display help
console.log('\n💡 Server Commands:');
console.log('• Press Ctrl+C to stop the server');
console.log('• Connect with: telnet localhost 8080');
console.log('• Or use the Rust client example: cargo run --example socket_client_usage');
console.log('');

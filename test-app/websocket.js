// Import required modules
const WebSocket = require('ws');
const http = require('http');

// Create an HTTP server
const server = http.createServer((req, res) => {
  res.writeHead(200, { 'Content-Type': 'text/plain' });
  res.end('WebSocket server is running');
});

// Create a WebSocket server by passing the HTTP server instance
const wss = new WebSocket.Server({ server });

// Handle WebSocket connections
wss.on('connection', (ws, req) => {
  const ip = req.socket.remoteAddress;
  console.log(`New connection from ${ip}`);

  // Send a welcome message to the client
  ws.send(JSON.stringify({ type: 'welcome', message: 'Connected to WebSocket server' }));

  // Handle messages from clients
  ws.on('message', (message) => {
    console.log(`Received: ${message}`);
    
    try {
      // Parse the message as JSON
      const parsedMessage = JSON.parse(message);
      
      // Echo the message back to the client
      ws.send(JSON.stringify({
        type: 'echo',
        content: parsedMessage,
        timestamp: new Date().toISOString()
      }));
    } catch (e) {
      // If the message is not valid JSON, send it back as is
      ws.send(JSON.stringify({
        type: 'echo',
        content: message.toString(),
        timestamp: new Date().toISOString()
      }));
    }
  });

  // Handle client disconnection
  ws.on('close', () => {
    console.log(`Connection from ${ip} closed`);
  });

  // Handle errors
  ws.on('error', (error) => {
    console.error(`Connection error: ${error.message}`);
  });
});

// Start the server
const PORT = process.env.PORT || 3004;
server.listen(PORT, () => {
  console.log(`WebSocket server is listening on port ${PORT}`);
});
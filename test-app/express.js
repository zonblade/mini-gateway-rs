// Import the express module
const express = require('express');

// Create an express application
const app = express();

// Define port (use environment variable PORT or default to 3000)
const PORT = process.env.PORT || 3002;

// Define a route for '/hello' endpoint
app.get('/hello', (req, res) => {
  res.json({ message: 'Hello, World!' });
});

// Start the server
app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`);
});
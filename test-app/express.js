// Import the express module
const express = require('express');

// Create an express application
const app = express();

// Add middleware to parse JSON
app.use(express.json());

// Define port (use environment variable PORT or default to 3000)
const PORT = process.env.PORT || 3002;

// Define a route for '/hello' endpoint
app.get('/hello', (req, res) => {
  res.json({ message: 'Hello, World!' });
});

// Define a new POST endpoint for '/combine'
app.post('/combine', (req, res) => {
  const { name, email } = req.body;

  // Validate input
  if (!name || !email) {
    return res.status(400).json({ error: 'Both name and email are required.' });
  }

  // Calculate the combined length of name and email
  const combinedLength = name.length + email.length;

  // Send the response as JSON
  res.json({ combinedLength });
});

// Start the server
app.listen(PORT, () => {
  console.log(`Server is running on http://localhost:${PORT}`);
});
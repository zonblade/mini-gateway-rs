/// Database abstraction module for SQLite operations.
///
/// This module provides a convenient abstraction layer over rusqlite for database operations.
/// It handles connection management, error handling, and provides a fluent query interface.
///
/// # Examples
///
/// ```rust
/// use router_api::module::database::{get_connection, DatabaseError};
///
/// fn main() -> Result<(), DatabaseError> {
///     // Get a database connection
///     let db = get_connection()?;
///
///     // Create a table if it doesn't exist
///     db.execute(
///         "CREATE TABLE IF NOT EXISTS users (
///             id INTEGER PRIMARY KEY AUTOINCREMENT,
///             username TEXT NOT NULL UNIQUE,
///             email TEXT NOT NULL
///         )",
///         [],
///     )?;
///
///     // Insert a record
///     db.execute(
///         "INSERT INTO users (username, email) VALUES (?1, ?2)",
///         ["johndoe", "john@example.com"],
///     )?;
///
///     // Query records
///     let users = db.query(
///         "SELECT id, username, email FROM users",
///         [],
///         |row| {
///             Ok((
///                 row.get::<_, i64>(0)?,
///                 row.get::<_, String>(1)?,
///                 row.get::<_, String>(2)?,
///             ))
///         },
///     )?;
///
///     for (id, username, email) in users {
///         println!("User {}: {} ({})", id, username, email);
///     }
///
///     Ok(())
/// }
/// ```
use rusqlite::{Connection, Result as SqliteResult};
use std::fs;
use std::path::Path;
use thiserror::Error;

/// Errors that can occur during database operations.
///
/// This enum provides a comprehensive set of errors that can occur when interacting
/// with the database, including SQLite-specific errors, I/O errors, and connection
/// management errors.
#[derive(Debug, Error)]
pub enum DatabaseError {
    /// An error from the underlying SQLite database.
    ///
    /// This variant wraps the original rusqlite error to preserve its context.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    
    /// An error from the file system.
    ///
    /// This can occur when creating directories or accessing database files.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// Error when attempting to access a database connection that has not been initialized.
    ///
    /// This typically occurs when there is a mutex poisoning or other threading issues.
    #[error("Database connection not initialized")]
    #[allow(dead_code)]
    NotInitialized,
    
    /// Custom error with a specific message.
    ///
    /// This is useful for domain-specific errors that are not directly
    /// related to SQLite or IO operations.
    #[error("{0}")]
    Custom(String),
}

impl DatabaseError {
    /// Creates a new DatabaseError with a custom message.
    ///
    /// This is a convenience method for creating a DatabaseError::Custom variant.
    ///
    /// # Parameters
    ///
    /// * `msg` - The custom error message
    ///
    /// # Returns
    ///
    /// A new DatabaseError::Custom instance
    pub fn from_msg<S: Into<String>>(msg: S) -> Self {
        DatabaseError::Custom(msg.into())
    }
}

/// Result type for database operations.
///
/// This type alias simplifies error handling by providing a consistent result type
/// for all database operations that includes the appropriate error type.
pub type DatabaseResult<T> = Result<T, DatabaseError>;

/// A simplified database wrapper for SQLite operations.
///
/// This implementation creates a new connection for each operation and
/// closes it immediately afterward, avoiding any potential locking issues
/// at the cost of a slight performance overhead. This is ideal for
/// low-concurrency applications where database locking is more problematic
/// than the overhead of creating new connections.
pub struct Database {
    /// Path to the SQLite database file
    db_path: String,
}

#[allow(dead_code)]
impl Database {
    /// Creates a new database connection to the main database.
    ///
    /// The database file is located at `/tmp/gwrs/data/core.sqlite`.
    ///
    /// # Returns
    ///
    /// A `DatabaseResult` containing either the new `Database` instance or an error
    /// if the database directory could not be created.
    pub fn new() -> DatabaseResult<Self> {
        // Ensure the directory exists
        let db_dir = Path::new("/tmp/gwrs/data");
        if !db_dir.exists() {
            fs::create_dir_all(db_dir)?;
        }
        
        let db_path = db_dir.join("core.sqlite").to_string_lossy().to_string();
        
        Ok(Self { db_path })
    }
    
    /// Creates a new database connection to the logging database.
    ///
    /// The database file is located at `/tmp/gwrs/data/core_logging.sqlite`.
    ///
    /// # Returns
    ///
    /// A `DatabaseResult` containing either the new `Database` instance or an error
    /// if the database directory could not be created.
    pub fn new_log() -> DatabaseResult<Self> {
        // Ensure the directory exists
        let db_dir = Path::new("/tmp/gwrs/data");
        if !db_dir.exists() {
            fs::create_dir_all(db_dir)?;
        }
        
        let db_path = db_dir.join("core_logging.sqlite").to_string_lossy().to_string();
        
        Ok(Self { db_path })
    }
    
    /// Creates a new connection to the database.
    ///
    /// This function is used internally to open a fresh connection for each operation.
    /// It also sets up necessary SQLite pragmas for better reliability.
    ///
    /// # Returns
    ///
    /// A `DatabaseResult` containing either the new Connection or an error
    fn connect(&self) -> DatabaseResult<Connection> {
        let conn = Connection::open(&self.db_path)?;
        
        // Configure SQLite for better reliability
        conn.execute_batch("
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA busy_timeout = 1000;
            PRAGMA foreign_keys = ON;
        ")?;
        
        Ok(conn)
    }
    
    /// Executes a raw SQL query with optional parameters.
    ///
    /// This method creates a new connection, executes the statement, and then
    /// automatically closes the connection.
    ///
    /// # Parameters
    ///
    /// * `sql` - The SQL statement to execute
    /// * `params` - The parameters to bind to the statement
    ///
    /// # Returns
    ///
    /// A `DatabaseResult` containing the number of rows modified by the statement
    /// or an error if the statement could not be executed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The SQL statement is invalid
    /// - A parameter binding fails
    /// - The statement execution fails
    ///
    /// # Examples
    ///
    /// ```rust
    /// use router_api::module::database::Database;
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Database::new()?;
    ///     
    ///     // Create a table
    ///     db.execute(
    ///         "CREATE TABLE IF NOT EXISTS products (
    ///             id INTEGER PRIMARY KEY,
    ///             name TEXT NOT NULL,
    ///             price REAL NOT NULL
    ///         )",
    ///         [],
    ///     )?;
    ///     
    ///     // Insert data
    ///     let rows_inserted = db.execute(
    ///         "INSERT INTO products (name, price) VALUES (?1, ?2)",
    ///         ["Product 1", &9.99.to_string()],
    ///     )?;
    ///     
    ///     println!("Inserted {} row(s)", rows_inserted);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn execute<P>(&self, sql: &str, params: P) -> DatabaseResult<usize> 
    where
        P: rusqlite::Params,
    {
        let conn = self.connect()?;
        let result = conn.execute(sql, params)?;
        Ok(result)
    }
    
    /// Executes a query and maps the results using the provided function.
    ///
    /// This method is used for SELECT statements that return multiple rows.
    /// The provided mapping function is called for each row, allowing you to
    /// transform the raw database row into your desired data structure.
    ///
    /// # Parameters
    ///
    /// * `sql` - The SQL query to execute
    /// * `params` - The parameters to bind to the query
    /// * `f` - A function that maps a database row to your desired type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that each row will be mapped to
    /// * `F` - The mapping function type
    /// * `P` - The parameter type
    ///
    /// # Returns
    ///
    /// A `DatabaseResult` containing a vector of mapped results
    /// or an error if the query could not be executed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The SQL statement is invalid
    /// - A parameter binding fails
    /// - The statement execution fails
    /// - The row mapping function returns an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use router_api::module::database::Database;
    ///
    /// #[derive(Debug)]
    /// struct Product {
    ///     id: i64,
    ///     name: String,
    ///     price: f64,
    /// }
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Database::new()?;
    ///     
    ///     let products = db.query(
    ///         "SELECT id, name, price FROM products WHERE price > ?1",
    ///         [5.0],
    ///         |row| {
    ///             Ok(Product {
    ///                 id: row.get(0)?,
    ///                 name: row.get(1)?,
    ///                 price: row.get(2)?,
    ///             })
    ///         },
    ///     )?;
    ///     
    ///     for product in products {
    ///         println!("Product: {:?}", product);
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn query<T, F, P>(&self, sql: &str, params: P, f: F) -> DatabaseResult<Vec<T>>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
        P: rusqlite::Params,
    {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(sql)?;
        let rows = stmt.query_map(params, f)?;
        
        let mut results = Vec::new();
        for row_result in rows {
            results.push(row_result?);
        }
        
        Ok(results)
    }
    
    /// Executes a query that returns a single result or None.
    ///
    /// This method is optimized for queries that should return at most one row,
    /// such as lookups by a unique identifier. It returns `None` if no matching
    /// row was found.
    ///
    /// # Parameters
    ///
    /// * `sql` - The SQL query to execute
    /// * `params` - The parameters to bind to the query
    /// * `f` - A function that maps a database row to your desired type
    ///
    /// # Type Parameters
    ///
    /// * `T` - The type that the row will be mapped to
    /// * `F` - The mapping function type
    /// * `P` - The parameter type
    ///
    /// # Returns
    ///
    /// A `DatabaseResult` containing an `Option<T>` (either the mapped row or `None`)
    /// or an error if the query could not be executed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The SQL statement is invalid
    /// - A parameter binding fails
    /// - The statement execution fails
    /// - The row mapping function returns an error
    ///
    /// # Examples
    ///
    /// ```rust
    /// use router_api::module::database::Database;
    ///
    /// #[derive(Debug)]
    /// struct User {
    ///     id: i64,
    ///     username: String,
    ///     email: String,
    /// }
    ///
    /// fn find_user_by_username(db: &Database, username: &str) -> Result<Option<User>, Box<dyn std::error::Error>> {
    ///     let user = db.query_one(
    ///         "SELECT id, username, email FROM users WHERE username = ?1",
    ///         [username],
    ///         |row| {
    ///             Ok(User {
    ///                 id: row.get(0)?,
    ///                 username: row.get(1)?,
    ///                 email: row.get(2)?,
    ///             })
    ///         },
    ///     )?;
    ///     
    ///     Ok(user)
    /// }
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Database::new()?;
    ///     
    ///     if let Some(user) = find_user_by_username(&db, "johndoe")? {
    ///         println!("Found user: {:?}", user);
    ///     } else {
    ///         println!("User not found");
    ///     }
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn query_one<T, F, P>(&self, sql: &str, params: P, f: F) -> DatabaseResult<Option<T>>
    where
        F: FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<T>,
        P: rusqlite::Params,
    {
        let conn = self.connect()?;
        let mut stmt = conn.prepare(sql)?;
        let mut rows = stmt.query_map(params, f)?;
        
        if let Some(row_result) = rows.next() {
            return Ok(Some(row_result?));
        }
        
        Ok(None)
    }
    
    /// Executes a function within a transaction.
    ///
    /// This method provides a convenient way to execute multiple statements within
    /// a single transaction, ensuring that they either all succeed or all fail.
    /// The transaction is automatically committed if the function returns `Ok`,
    /// or rolled back if it returns `Err`.
    ///
    /// # Parameters
    ///
    /// * `f` - A function that takes a reference to a transaction and returns a result
    ///
    /// # Type Parameters
    ///
    /// * `T` - The return type of the function
    /// * `F` - The function type
    ///
    /// # Returns
    ///
    /// A `DatabaseResult` containing the result of the function or an error
    /// if the transaction could not be executed.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The transaction could not be started
    /// - The function returns an error
    /// - The transaction could not be committed
    ///
    /// # Examples
    ///
    /// ```rust
    /// use router_api::module::database::Database;
    ///
    /// fn transfer_funds(
    ///     db: &Database,
    ///     from_account: &str,
    ///     to_account: &str,
    ///     amount: f64,
    /// ) -> Result<(), Box<dyn std::error::Error>> {
    ///     db.transaction(|conn| {
    ///         // Deduct from the source account
    ///         conn.execute(
    ///             "UPDATE accounts SET balance = balance - ?1 WHERE account_number = ?2",
    ///             [amount, from_account],
    ///         )?;
    ///         
    ///         // Add to the destination account
    ///         conn.execute(
    ///             "UPDATE accounts SET balance = balance + ?1 WHERE account_number = ?2",
    ///             [amount, to_account],
    ///         )?;
    ///         
    ///         // Log the transaction
    ///         conn.execute(
    ///             "INSERT INTO transactions (from_account, to_account, amount, timestamp) 
    ///              VALUES (?1, ?2, ?3, datetime('now'))",
    ///             [from_account, to_account, amount],
    ///         )?;
    ///         
    ///         Ok(())
    ///     })?;
    ///     
    ///     Ok(())
    /// }
    ///
    /// fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let db = Database::new()?;
    ///     transfer_funds(&db, "ACC001", "ACC002", 100.0)?;
    ///     println!("Transfer successful");
    ///     Ok(())
    /// }
    /// ```
    pub fn transaction<T, F>(&self, f: F) -> DatabaseResult<T>
    where
        F: FnOnce(&Connection) -> SqliteResult<T>,
    {
        let mut conn = self.connect()?;
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
    
    /// Checks if a table exists and has the expected columns
    ///
    /// This is a simple utility method to check if a table exists with its expected structure.
    /// It returns true only if the table exists and all specified columns are present.
    ///
    /// # Parameters
    ///
    /// * `table_name` - The name of the table to check
    /// * `expected_columns` - Array of column names that should exist in the table
    ///
    /// # Returns
    ///
    /// * `Ok(true)` - If the table exists with all expected columns
    /// * `Ok(false)` - If the table doesn't exist or is missing columns
    /// * `Err(DatabaseError)` - If there was a database error during the check
    ///
    /// # Example
    ///
    /// ```rust
    /// let db = get_connection()?;
    /// if !db.table_exists_with_columns("users", &["id", "username", "email"])? {
    ///     // Create or fix the table
    /// }
    /// ```
    pub fn table_exists_with_columns(
        &self, 
        table_name: &str, 
        expected_columns: &[&str]
    ) -> DatabaseResult<bool> {
        let conn = self.connect()?;
        
        // First check if the table exists
        let table_exists: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?",
                [table_name],
                |row| row.get::<_, i64>(0)
            )
            .map(|count| count > 0)
            .map_err(DatabaseError::from)?;
            
        if !table_exists {
            return Ok(false);
        }
        
        // Then check if the table has the expected columns
        for column_name in expected_columns {
            let column_exists: bool = conn
                .query_row(
                    "SELECT COUNT(*) FROM pragma_table_info(?) WHERE name=?",
                    rusqlite::params![table_name, column_name],
                    |row| row.get::<_, i64>(0)
                )
                .map(|count| count > 0)
                .map_err(DatabaseError::from)?;
                
            if !column_exists {
                return Ok(false);
            }
        }
        
        // Run a quick integrity check on the table
        let result = conn.query_row(
            "PRAGMA quick_check", 
            [],
            |row| row.get::<_, String>(0)
        );
        
        match result {
            Ok(ok_msg) if ok_msg == "ok" => Ok(true),
            Ok(_) => Ok(false),  // If we get any message other than "ok", table is corrupt
            Err(_) => Ok(false), // Error during quick_check indicates issues with the table
        }
    }
}

/// A builder pattern for constructing SQL queries with type safety.
///
/// The `Query` struct helps in building parameterized SQL queries with type
/// safety. It allows for fluent chaining of parameters and ensures that
/// the expected return type is maintained throughout the query construction.
///
/// # Type Parameters
///
/// * `T` - The expected result type after mapping the query results
///
/// # Examples
///
/// ```rust
/// use router_api::module::database::{Database, Query};
///
/// struct User {
///     id: i64,
///     username: String,
///     email: String,
/// }
///
/// fn find_users_by_email_domain(db: &Database, domain: &str) -> Result<Vec<User>, Box<dyn std::error::Error>> {
///     let query = Query::<User>::new("SELECT id, username, email FROM users WHERE email LIKE ?")
///         .param(format!("%@{}", domain));
///
///     // The query can be executed when the Database implementation supports it
///     // let users = db.execute_query(query)?;
///     
///     // For now, we'll just show the concept
///     let _ = query;
///     
///     // Placeholder implementation
///     Ok(vec![])
/// }
///
/// fn main() {
///     // Example usage
/// }
/// ```
#[allow(dead_code)]
pub struct Query<T> {
    /// The SQL query string.
    sql: String,
    
    /// The parameters to bind to the query.
    params: Vec<Box<dyn rusqlite::ToSql>>,
    
    /// Phantom data to maintain the type parameter.
    _marker: std::marker::PhantomData<T>,
}

#[allow(dead_code)]
impl<T> Query<T> {
    /// Creates a new query with the given SQL statement.
    ///
    /// # Parameters
    ///
    /// * `sql` - The SQL query string
    ///
    /// # Returns
    ///
    /// A new `Query` instance with the given SQL statement
    ///
    /// # Examples
    ///
    /// ```rust
    /// use router_api::module::database::Query;
    ///
    /// struct Product {
    ///     id: i64,
    ///     name: String,
    ///     price: f64,
    /// }
    ///
    /// let query = Query::<Product>::new("SELECT id, name, price FROM products WHERE price > ?");
    /// ```
    pub fn new(sql: &str) -> Self {
        Self {
            sql: sql.to_string(),
            params: Vec::new(),
            _marker: std::marker::PhantomData,
        }
    }
    
    /// Adds a parameter to the query.
    ///
    /// This method adds a parameter to the query and returns the modified
    /// query builder, allowing for method chaining.
    ///
    /// # Parameters
    ///
    /// * `param` - The parameter to add to the query
    ///
    /// # Type Parameters
    ///
    /// * `P` - The type of the parameter, which must implement `ToSql`
    ///
    /// # Returns
    ///
    /// The modified `Query` instance with the new parameter added
    ///
    /// # Examples
    ///
    /// ```rust
    /// use router_api::module::database::Query;
    ///
    /// struct User {
    ///     id: i64,
    ///     username: String,
    ///     age: i32,
    /// }
    ///
    /// let query = Query::<User>::new("SELECT id, username, age FROM users WHERE age > ? AND username LIKE ?")
    ///     .param(18)
    ///     .param("john%");
    /// ```
    pub fn param<P: rusqlite::ToSql + 'static>(mut self, param: P) -> Self {
        self.params.push(Box::new(param));
        self
    }
}

/// Creates and returns a new database connection.
///
/// This is a convenience function that creates a new `Database` instance
/// and returns it, making it easier to get a database connection without
/// having to explicitly call `Database::new()`.
///
/// # Returns
///
/// A `DatabaseResult` containing either the new `Database` instance or an error
/// if the connection could not be established.
///
/// # Errors
///
/// This function will return an error if:
/// - The directory structure could not be created
/// - The database file could not be opened or created
///
/// # Examples
///
/// ```rust
/// use router_api::module::database::{get_connection, DatabaseError};
///
/// fn main() -> Result<(), DatabaseError> {
///     let db = get_connection()?;
///     println!("Database connection established");
///     Ok(())
/// }
/// ```
pub fn get_connection() -> DatabaseResult<Database> {
    Database::new()
}

pub fn get_connection_log() -> DatabaseResult<Database> {
    Database::new_log()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    /// Tests the database connection and basic CRUD operations.
    ///
    /// This test:
    /// 1. Creates a database connection
    /// 2. Creates a test table
    /// 3. Inserts a row
    /// 4. Queries the row
    /// 5. Asserts that the query returned the expected result
    /// 6. Cleans up by dropping the table
    #[test]
    fn test_database_connection() {
        let db = Database::new().expect("Failed to connect to database");
        
        // Create a test table
        db.execute(
            "CREATE TABLE IF NOT EXISTS test_table (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
            [],
        ).expect("Failed to create test table");
        
        // Insert data
        db.execute(
            "INSERT INTO test_table (name) VALUES (?1)",
            ["Test Name"],
        ).expect("Failed to insert data");
        
        // Query data
        let results = db.query(
            "SELECT id, name FROM test_table WHERE name = ?1",
            ["Test Name"],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        ).expect("Failed to query data");
        
        assert!(!results.is_empty());
        
        // Clean up
        db.execute("DROP TABLE test_table", []).expect("Failed to drop test table");
    }
}
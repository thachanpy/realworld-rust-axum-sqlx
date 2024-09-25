# Real world Rust Axum SQLX

Our Rust [Rust](https://www.rust-lang.org/) API template, powered by [Axum](https://docs.rs/axum/latest/axum/), provides a
robust framework for building high-performance web APIs. With this template, you can quickly develop RESTul APIs with
minimal boilerplate code, leveraging Axum's powerful features such as middleware, routing, and error handling.
Whether you're prototyping a new project or developing a production-grade API, this template accelerates development
while maintaining performance and reliability.

## Authors

- [Thach An](https://github.com/thachanpy)

## Support features

### API and Worker Mode
- Separation of concerns between API handling and background task processing.
- Independent scaling of API servers and workers.

### Middlewares
- CORS:
    - Allow cross-origin requests from specified domains.
- Authentication with JWT:
    - Secure authentication using JSON Web Tokens.
    - Tokens are signed with HMAC or RSA algorithms.
    - Includes support for token expiration and refresh tokens.
- Authorization with Security Roles (User, Admin):
    - Role-based access control to restrict endpoints.
    - Middleware to check user roles before accessing resources.
- Compress Layer:
    - Middleware to compress HTTP responses.
- Logging:
    - Structured logging with `tracing` crate.
    - Different logging levels (info, debug, error).
- Error Handling:
    - Centralized error handling middleware.
    - Custom error responses with detailed messages.
    - Logging of errors for diagnostics.

### Dependencies Injection using `state` Mode
- DI for managing application state and dependencies.
- Ensures testability and modularity.

### Database
- PostgreSQL with `sqlx` and `sea-query` (code generator):
    - Database communication using async `sqlx`.
    - Query builder with `sea-query` for type-safety.
    - Connection pooling and transaction management.
- Database Migration with `flyway`:
    - Automated and versioned database migration scripts.
    - Easy rollback and forward capabilities.
- Repository Layer to Adapt to DB:
    - Abstraction layer for database operations.
    - Ensures separation of concerns and ease of testing.
- Enable `replicas` mode.
- Support `pagination` and `sorting` requests.

### Authentication
- Basic Sign Up/In with Username and Password:
    - Endpoints for user registration and login.
    - Secure password storage with hashing.
- OAuth2 with Google:
    - Integration with Google's OAuth2 for authentication.
    - Support for obtaining user data from Google after authentication.
- Sign-Out:
    - Endpoint to manage user logout.
    - Invalidation of tokens and clearing sessions.

### AWS Services
- S3 for Storage:
    - File upload using S3.
- SQS for Job Producer, Consumer, and Processor:
    - Queue management using AWS SQS.
    - Producers to send messages, consumers to process them.

### Dynamic Configuration
- Application Configuration with `application.yaml` and Environment Variable Overwrite:
    - Config file for default settings.
    - Use specific environment config file by following the convention `application-<ENVIRONMENT>.yaml` 
      to overwrite the base configurations.
    - Environment variables to override specific configurations

### Unit Tests and Integration Tests
- Comprehensive test cases covering core functionality.
- Integration tests for end-to-end scenario validation.


### Dockerization
- Dockerfiles and Docker Compose for containerization.


## Code Structure

```
.
├── .editorconfig
├── .github
│   └── workflows
├── .gitignore
├── .tools
├── Cargo.lock
├── Cargo.toml
├── Makefile
├── README.md
├── docker-compose.yml
├── rustfmt.toml
└── src
    ├── .deploy
    │   ├── dockerfiles
    │   └── k8s
    ├── api
    │   ├── client
    │   ├── controller
    │   ├── manager
    │   │   ├── auth
    │   │   └── users
    │   ├── repository
    │   │    └── users
    │   ├── state
    │   │   ├── auth
    │   │   └── users
    │   └── test
    ├── core
    │   ├── cors
    │   ├── error
    │   ├── logging
    │   ├── response
    │   └── security
    ├── db
    │   └── sql
    ├── job
    │   ├── consumer
    │   ├── event
    │   └── processor
    ├── resources
    │   └── application.yaml
    ├── service
    │   ├── aws
    │   └── oauth2
    ├── utils
    └── worker
```

- `.editorconfig`: Configuration file for defining and maintaining consistent coding styles across different editors and
  IDEs.
- `.github`: Contains GitHub-specific configurations and workflows.
- `workflows`: Contains GitHub Actions workflow files for CI/CD, automation, and other GitHub Actions tasks.
- `.gitignore`: Specifies which files and directories Git should ignore. Useful for excluding build artifacts,
  dependencies, and sensitive information.
- `.tools`: This directory might be used for tools or scripts specific to your project that assist in development or
  operations.
- `Cargo.lock`: Auto-generated file by Cargo (Rust package manager) that locks dependencies to specific versions.
  Ensures reproducible builds.
- `Cargo.toml`: The manifest file for Rust projects, listing dependencies, project metadata, and build configuration.
- `Makefile`: Defines a set of directives used with the make build automation tool. Useful for defining custom build
  commands, scripts, or tasks.
- `README.md`: Documentation file that provides an overview of the project, setup instructions, usage, and other
  relevant information.
- `docker-compose.yml`: Configuration file for Docker Compose, defining services, networks, and volumes for
  multi-container Docker applications.
- `rustfmt.toml`: Configuration file for rustfmt, a tool for formatting Rust code according to style guidelines.
- `src`: This is the main source directory containing all the application code.
    - `.deploy`: Contains deployment-related configurations and files.
      - `dockerfiles`: Directory for Dockerfile(s) used to build Docker images for your application.
      - `k8s`: Contains Kubernetes configuration files for deploying and managing your application in a Kubernetes
        cluster.
    - `api`: This directory handles the API-related code.
      - `client`: Likely contains code for interacting with APIs or services.
      - `controller`: Contains the controllers that handle incoming requests, process them, and return responses.
      - `manager`: Manages various aspects of the application’s state or business logic.
      - `repository`: Handles data access and persistence, interacting with the database or other storage mechanisms.
      - `state`: Manages the application’s state, possibly including configurations or shared data.
      - `test`: Contains tests for the API layer, ensuring the correctness of the APIs functionality.
    - `core`: Contains core functionalities and utilities used throughout the application.
      - `cors`: Code for handling Cross-Origin Resource Sharing (CORS) settings and policies.
      - `error`: Defines error handling mechanisms and custom error types.
      - `logging`: Manages logging configurations and utilities.
      - `response`: Handles HTTP response formatting and utilities.
      - `security`: Contains security-related functionality, such as authentication and authorization.
    - `db`: Deals with database-related code.
      - `sql`: Likely contains SQL queries or database schema definitions.
    - `job`: This directory handles the Job message queue.
      - `consumer`: Manages consumers that retrieve and process jobs or messages from external sources like message queues (e.g., SQS).
      - `event`: Defines the structure or types of events that can be processed within the system.
      - `processor`: Contains job processing logic that defines how specific jobs or tasks are processed once retrieved by the consumer.
    - `resources`: This might contain configuration files or other resources needed by the application.
    - `services`: Contains the internal or the 3rd services such as AWS or Google OAuth2.
    - `utils`: Contains utility functions and helper methods used across the application.
    - `worker`: Likely contains background workers or tasks that run independently of user requests, potentially handling asynchronous tasks.

## Environment Variables

- To run this project, you can to add the following environment variables to replace the `application.yaml` place holders.

```
...
server:
  address: ${SERVER_ADDRESS:0.0.0.0}
  api:
    port: ${SERVER_API_PORT:8080}
  cors:
    allowed_origin: ${SERVER_CORS_ALLOWED_ORIGIN:"*"}
...
```
```bash
export SERVER_ADDRESS=127.0.0.1 # overwrite `0.0.0.0` default value
```
- Use the `ENVIRONMENT=<name>` variable to map to a specific configuration file, such as `application-<name>.yaml`, based on the environment name.”

## Local Development

### Prerequisites

- Install [RustRover](https://www.jetbrains.com/rust) or [VsCode](https://code.visualstudio.com)
- Install [Homebrew](https://brew.sh)
- Install `brew install flyway` for SQL migration. If you have an `M1` laptop, make sure to add flyway under your bin
  path, i.e. `/usr/bin/flyway`
- Install [OrbStack](https://orbstack.dev) for containers.
- Setup AWS Localstack
    ```bash
    FILE_PATH="$HOME/.aws/config"
    
    CONTENT="
    [profile localstack]
    aws_access_key_id=dummy
    aws_secret_access_key=dummy
    endpoint_url=http://localhost.localstack.cloud:4566
    region=us-west-2
    output=json
    "
    echo "$CONTENT" >> "$FILE_PATH"
    ```
### Getting Started

#### Clone the project

```bash
git clone git@github.com:thachanpy/realworld-rust-axum-sqlx.git
```

#### Go to the project directory

```bash
cd realworld-rust-axum-sqlx
```

#### Install the binaries

```bash
make install
```

#### Bring up the 3rd containers like db, localstack,...

```bash
make up-docker-compose
```

#### Run the Flyway migration

```bash
make flyway-migration
```

#### Create the localstack resources

```bash
make aws
```

#### Bring all infra dependencies
```bash
make up-infra
```

#### Shutdown all infra dependencies
```bash
make down-infra
```

#### Start the server

```bash
make start
```

## Running Tests

### Check the code format

```bash
make fmt-check
```

### Test the application

```bash
make test
```

## 🔗 Links

- [rust](https://www.rust-lang.org/)
- [axum](https://docs.rs/axum/latest/axum/)
- [sqlx](https://docs.rs/sqlx/latest/sqlx/)
- [sea-query](https://crates.io/crates/sea-query)

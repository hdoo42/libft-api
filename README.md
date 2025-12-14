
# ft-api

A Rust library for interacting with the 42 API. This repository provides the `libft-api` crate for core API client functionality

## Project Structure

* `libft-api/`: The main library crate that provides the 42 API client and related functionalities.
    * `src/`: Contains the source code for the library.
        * `api/`: Modules for different API endpoints (campus, cursus, exam, group, project, project_session, project_user, scale_team, user).
        * `models/`: Structs representing data returned by the 42 API.
        * `auth.rs`: Handles authentication with the 42 API.
        * `connector.rs`: Manages HTTP connections to the API.
        * `common.rs`: Common utilities, error handling, and parameter types.
        * `info.rs`: Contains constant values like Cursus IDs and Campus IDs.
    * `bin/`: Contains example applications demonstrating the usage of `libft-api`. Examples include:
        * `blackholed.rs`: Fetches user data based on various criteria.
        * `campus_users.rs`: Fetches and processes campus user project data.
        * `evaluation.rs`: Fetches evaluation historics and scale team data.
        * `exam_resubscribe.rs`: Example for resubscribing users to an exam.
        * `final_score.rs`: Calculates and outputs final scores based on team and scale team data.
        * `get_user_ext.rs`: Fetches extended user information.
        * `journals.rs`: Fetches journal entries for a campus.
        * `locations.rs`: Fetches location data for a campus.
        * `project_stats.rs`: Fetches statistics for specific projects.
        * `teams.rs`: Example for posting multiple scale teams.
        * `user_creation.rs`: Example for creating test user accounts.
        * `user_subscribe.rs`: Example for subscribing users to projects and exams.
    * `README.md`: This file, providing an overview of the `libft-api` crate.
* `libft-api-derive/`: A procedural macro crate. Currently, it includes a simple `add` function.
* `.github/workflows/rust.yml`: GitHub Actions workflow for continuous integration, including building, checking, and testing the Rust code.

## Features

The `libft-api` crate provides Rust bindings for various 42 API v2 endpoints, including:

* **Authentication**: Handles OAuth2 token generation and management.
* **Campus**:
    * Fetch campus information.
    * Fetch campus journals.
    * Fetch campus locations.
    * Fetch campus users.
* **Cursus**:
    * Fetch projects for a specific cursus.
* **Exams**:
    * Fetch exam information.
    * Subscribe users to exams.
* **Groups**:
    * Fetch group information.
    * Add users to groups.
* **Projects**:
    * Fetch project data.
    * Fetch project details.
    * Fetch teams for a project.
* **Project Sessions**:
    * Fetch scale teams for a project session.
    * Fetch teams for a project session.
* **Project Users**:
    * Fetch project user information.
    * Subscribe users to projects.
* **Scale Teams**:
    * Fetch scale team information.
    * Create multiple scale teams.
* **Users**:
    * Fetch user information (basic and extended).
    * Create users.
    * Fetch user correction point history.
    * Add correction points to users.
    * Fetch cursus users for a user.
    * Add users to a cursus.
    * Fetch user locations and location statistics.
    * Fetch project users for a user.
    * Fetch teams for a user.

## Getting Started

### Prerequisites

* Rust (latest nightly is used in workflows, but stable should generally work).
* You need to set the following environment variables with your 42 API client credentials:
    * `FT_API_CLIENT_UID`
    * `FT_API_CLIENT_SECRET`

### Installation

To use `libft-api` in your Rust project, add it as a dependency in your `Cargo.toml` file.

```toml
[dependencies]
libft-api = { git = "[https://github.com/hdoo42/ft-api.git](https://github.com/hdoo42/ft-api.git)" } # Or specify a version if tags are used
````

## Example

Create a token -\> Create a client -\> Create a session (simple wrapper) -\> Send API requests\!

```rust
    //build a token
    let res = FtApiToken::build_from_env().await; // Ensure AuthInfo::build_from_env() is called if necessary

    if let Ok(token) = res {
        println!("token ok");
        let client = FtClient::new(FtClientReqwestConnector::with_connector(
            reqwest::Client::new(),
        ));

        let session = client.open_session(&token);
        // Example: Fetching campus locations for Gyeongsan (campus ID 69)
        // Note: Ensure GYEONGSAN is correctly defined or use FtCampusId::new(69)
        let res = session.campus_id_locations(
            FtApiCampusIdLocationsRequest::new(FtCampusId::new(69)) // Assuming GYEONGSAN = 69
        ).await?;
      // res will contain all the locations for campus gs(Gyeongsan)
    }
```

## API Endpoints Implemented

The library aims to cover the 42 API. Here's a summary of what's currently implemented:

  * **OAuth**: Token generation and management.
  * **V2 API Endpoints**:
      * `campus/:campus_id/locations`
      * `campus/:campus_id/users`
      * `campus/:campus_id/journals`
      * `campus_users` (and `users/:user_id/campus_users`)
      * `cursus/:cursus_id/projects`
      * `exams` (GET) and `exams/:exam_id/exams_users` (POST)
      * `groups` (GET) and `groups_users` (POST)
      * `project_data`
      * `projects`
      * `projects/:project_id/teams`
      * `project_sessions/:project_session_id/scale_teams`
      * `project_sessions/:project_session_id/teams`
      * `projects_users` (GET and POST)
      * `scale_teams` (GET) and `scale_teams/multiple_create` (POST)
      * `users` (GET and POST)
      * `users/:id` (GET, for extended user info)
      * `users/:user_id/correction_point_historics`
      * `users/:id/correction_points/add` (POST)
      * `users/:user_id/cursus_users` (GET and POST)
      * `users/:user_id/locations`
      * `users/:user_id/locations_stats`
      * `users/:user_id/projects_users`
      * `users/:user_id/teams`

## Plans

There are two major components that need to be implemented further or refined:

1.  **API Request Implementation**: This involves setting up the functions and methods necessary to send requests to the 42 API. More endpoints can be added.
2.  **Data Structures for API Responses**: This entails defining Rust structs that will map to the JSON data returned by the 42 API. These structures will be used to deserialize the API responses into Rust objects. More response fields and structures can be added for completeness.

## Workflows

This repository uses GitHub Actions for continuous integration. The workflow is defined in `.github/workflows/rust.yml` and includes the following steps:

  * **Trigger**: On push or pull request to the `master` branch.
  * **Environment**: Runs on `ubuntu-latest`.
  * **Steps**:
    1.  Checkout code.
    2.  Install the latest nightly Rust toolchain with `rustfmt` and `clippy` components.
    3.  Run `cargo check`.
    4.  Run `cargo build --verbose`.
    5.  Run `cargo test --verbose`. Tests require `FT_API_CLIENT_UID` and `FT_API_CLIENT_SECRET` secrets.

## Contribute?

Contributions are very welcome.

Let me know if you need any more help\!

## License

The license for this project is not specified in the provided files. It's recommended to add a `LICENSE` file to the repository.

// tests/mocks/github_api.rs
// Mocks for GitHub API responses

use serde::{Deserialize, Serialize};
use mockito::{self, Mock, Server};

/// Sets up a mock server for GitHub API requests
pub fn setup_mock_server() -> Server {
    mockito::Server::new()
}

/// Creates a mock for GET /repos/{owner}/{repo}/collaborators
pub fn mock_collaborators(server: &Server, owner: &str, repo: &str) -> Mock {
    let response_body = format!(r#"[
        {{
            "login": "test-user1",
            "id": 12345,
            "node_id": "MDQ6VXNlcjEyMzQ1",
            "avatar_url": "https://avatars.githubusercontent.com/u/12345?v=4",
            "gravatar_id": "",
            "url": "https://api.github.com/users/test-user1",
            "html_url": "https://github.com/test-user1",
            "followers_url": "https://api.github.com/users/test-user1/followers",
            "following_url": "https://api.github.com/users/test-user1/following{{/other_user}}",
            "gists_url": "https://api.github.com/users/test-user1/gists{{/gist_id}}",
            "starred_url": "https://api.github.com/users/test-user1/starred{{/owner}}{{/repo}}",
            "subscriptions_url": "https://api.github.com/users/test-user1/subscriptions",
            "organizations_url": "https://api.github.com/users/test-user1/orgs",
            "repos_url": "https://api.github.com/users/test-user1/repos",
            "events_url": "https://api.github.com/users/test-user1/events{{/privacy}}",
            "received_events_url": "https://api.github.com/users/test-user1/received_events",
            "type": "User",
            "site_admin": false,
            "name": "Test User 1",
            "email": "test-user1@example.com"
        }},
        {{
            "login": "test-user2",
            "id": 67890,
            "node_id": "MDQ6VXNlcjY3ODkw",
            "avatar_url": "https://avatars.githubusercontent.com/u/67890?v=4",
            "gravatar_id": "",
            "url": "https://api.github.com/users/test-user2",
            "html_url": "https://github.com/test-user2",
            "followers_url": "https://api.github.com/users/test-user2/followers",
            "following_url": "https://api.github.com/users/test-user2/following{{/other_user}}",
            "gists_url": "https://api.github.com/users/test-user2/gists{{/gist_id}}",
            "starred_url": "https://api.github.com/users/test-user2/starred{{/owner}}{{/repo}}",
            "subscriptions_url": "https://api.github.com/users/test-user2/subscriptions",
            "organizations_url": "https://api.github.com/users/test-user2/orgs",
            "repos_url": "https://api.github.com/users/test-user2/repos",
            "events_url": "https://api.github.com/users/test-user2/events{{/privacy}}",
            "received_events_url": "https://api.github.com/users/test-user2/received_events",
            "type": "User",
            "site_admin": false,
            "name": null,
            "email": null
        }}
    ]"#);

    server.mock("GET", &format!("/repos/{}/{}/collaborators", owner, repo))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for GET /repos/{owner}/{repo}/issues
pub fn mock_issues(server: &Server, owner: &str, repo: &str, state: Option<&str>) -> Mock {
    let state_param = match state {
        Some(s) => format!("?state={}", s),
        None => "".to_string(),
    };

    let response_body = format!(r#"[
        {{
            "id": 1234567,
            "node_id": "MDExOlB1bGxSZXF1ZXN0MTIzNDU2Nw==",
            "number": 1,
            "state": "open",
            "title": "Test issue 1",
            "user": {{
                "login": "test-user1",
                "id": 12345
            }},
            "labels": [
                {{
                    "id": 123456789,
                    "name": "bug",
                    "color": "fc2929"
                }}
            ],
            "assignees": [
                {{
                    "login": "test-user2",
                    "id": 67890
                }}
            ],
            "comments": 5,
            "created_at": "2023-01-01T00:00:00Z",
            "updated_at": "2023-01-02T00:00:00Z",
            "closed_at": null,
            "body": "This is a test issue",
            "locked": false,
            "html_url": "https://github.com/{}/{}/issues/1"
        }},
        {{
            "id": 7654321,
            "node_id": "MDExOlB1bGxSZXF1ZXN0NzY1NDMyMQ==",
            "number": 2,
            "state": "closed",
            "title": "Test issue 2",
            "user": {{
                "login": "test-user2",
                "id": 67890
            }},
            "labels": [],
            "assignees": [],
            "comments": 0,
            "created_at": "2023-01-03T00:00:00Z",
            "updated_at": "2023-01-04T00:00:00Z",
            "closed_at": "2023-01-04T00:00:00Z",
            "body": null,
            "locked": false,
            "html_url": "https://github.com/{}/{}/issues/2"
        }}
    ]"#, owner, repo, owner, repo);

    server.mock("GET", &format!("/repos/{}/{}/issues{}", owner, repo, state_param))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for GET /repos/{owner}/{repo}/pulls
pub fn mock_pull_requests(server: &Server, owner: &str, repo: &str, state: Option<&str>) -> Mock {
    let state_param = match state {
        Some(s) => format!("?state={}", s),
        None => "".to_string(),
    };

    let response_body = format!(r#"[
        {{
            "id": 987654321,
            "node_id": "MDExOlB1bGxSZXF1ZXN0OTg3NjU0MzIx",
            "number": 3,
            "state": "open",
            "title": "Test PR 1",
            "user": {{
                "login": "test-user1",
                "id": 12345
            }},
            "body": "This is a test PR",
            "created_at": "2023-01-05T00:00:00Z",
            "updated_at": "2023-01-06T00:00:00Z",
            "closed_at": null,
            "merged_at": null,
            "merge_commit_sha": null,
            "assignees": [],
            "requested_reviewers": [],
            "labels": [
                {{
                    "id": 123456789,
                    "name": "enhancement",
                    "color": "84b6eb"
                }}
            ],
            "draft": false,
            "merged": false,
            "mergeable": true,
            "comments": 2,
            "commits": 3,
            "additions": 50,
            "deletions": 20,
            "changed_files": 5,
            "html_url": "https://github.com/{}/{}/pull/3"
        }},
        {{
            "id": 123456789,
            "node_id": "MDExOlB1bGxSZXF1ZXN0MTIzNDU2Nzg5",
            "number": 4,
            "state": "closed",
            "title": "Test PR 2",
            "user": {{
                "login": "test-user2",
                "id": 67890
            }},
            "body": null,
            "created_at": "2023-01-07T00:00:00Z",
            "updated_at": "2023-01-08T00:00:00Z",
            "closed_at": "2023-01-08T00:00:00Z",
            "merged_at": "2023-01-08T00:00:00Z",
            "merge_commit_sha": "abcdef1234567890",
            "assignees": [],
            "requested_reviewers": [],
            "labels": [],
            "draft": false,
            "merged": true,
            "mergeable": null,
            "comments": 0,
            "commits": 1,
            "additions": 10,
            "deletions": 5,
            "changed_files": 1,
            "html_url": "https://github.com/{}/{}/pull/4"
        }}
    ]"#, owner, repo, owner, repo);

    server.mock("GET", &format!("/repos/{}/{}/pulls{}", owner, repo, state_param))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for GET /repos/{owner}/{repo}/pulls/{pull_number}/reviews
pub fn mock_code_reviews(server: &Server, owner: &str, repo: &str, pull_number: u32) -> Mock {
    let response_body = format!(r#"[
        {{
            "id": 12345,
            "node_id": "MDE3OlB1bGxSZXF1ZXN0UmV2aWV3MTIzNDU=",
            "user": {{
                "login": "test-user2",
                "id": 67890
            }},
            "body": "Great changes!",
            "state": "APPROVED",
            "html_url": "https://github.com/{}/{}/pull/{}/reviews/12345",
            "pull_request_url": "https://api.github.com/repos/{}/{}/pulls/{}",
            "submitted_at": "2023-01-06T12:00:00Z",
            "commit_id": "abcdef1234567890"
        }},
        {{
            "id": 67890,
            "node_id": "MDE3OlB1bGxSZXF1ZXN0UmV2aWV3Njc4OTA=",
            "user": {{
                "login": "test-user1",
                "id": 12345
            }},
            "body": "Please fix these issues",
            "state": "CHANGES_REQUESTED",
            "html_url": "https://github.com/{}/{}/pull/{}/reviews/67890",
            "pull_request_url": "https://api.github.com/repos/{}/{}/pulls/{}",
            "submitted_at": "2023-01-06T13:00:00Z",
            "commit_id": "abcdef1234567890"
        }}
    ]"#, owner, repo, pull_number, owner, repo, pull_number, owner, repo, pull_number, owner, repo, pull_number);

    server.mock("GET", &format!("/repos/{}/{}/pulls/{}/reviews", owner, repo, pull_number))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for GET /repos/{owner}/{repo}/issues/comments
pub fn mock_issue_comments(server: &Server, owner: &str, repo: &str) -> Mock {
    let response_body = format!(r#"[
        {{
            "id": 123456,
            "node_id": "MDEyOklzc3VlQ29tbWVudDEyMzQ1Ng==",
            "url": "https://api.github.com/repos/{}/{}/issues/comments/123456",
            "html_url": "https://github.com/{}/{}/issues/1#issuecomment-123456",
            "body": "This is an issue comment",
            "user": {{
                "login": "test-user1",
                "id": 12345
            }},
            "created_at": "2023-01-02T10:00:00Z",
            "updated_at": "2023-01-02T10:00:00Z",
            "issue_url": "https://api.github.com/repos/{}/{}/issues/1"
        }},
        {{
            "id": 654321,
            "node_id": "MDEyOklzc3VlQ29tbWVudDY1NDMyMQ==",
            "url": "https://api.github.com/repos/{}/{}/issues/comments/654321",
            "html_url": "https://github.com/{}/{}/issues/1#issuecomment-654321",
            "body": "Another issue comment",
            "user": {{
                "login": "test-user2",
                "id": 67890
            }},
            "created_at": "2023-01-02T11:00:00Z",
            "updated_at": "2023-01-02T11:00:00Z",
            "issue_url": "https://api.github.com/repos/{}/{}/issues/1"
        }}
    ]"#, owner, repo, owner, repo, owner, repo, owner, repo, owner, repo, owner, repo);

    server.mock("GET", &format!("/repos/{}/{}/issues/comments", owner, repo))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for GET /repos/{owner}/{repo}/pulls/comments
pub fn mock_pull_request_comments(server: &Server, owner: &str, repo: &str) -> Mock {
    let response_body = format!(r#"[
        {{
            "id": 789012,
            "node_id": "MDI0OlB1bGxSZXF1ZXN0UmV2aWV3Q29tbWVudDc4OTAxMg==",
            "pull_request_review_id": 12345,
            "diff_hunk": "@@ -1,3 +1,4 @@\\n+// New comment\\n function test() {{\\n   return true;\\n }}",
            "path": "src/main.rs",
            "position": 1,
            "original_position": 1,
            "commit_id": "abcdef1234567890",
            "original_commit_id": "0987654321fedcba",
            "user": {{
                "login": "test-user1",
                "id": 12345
            }},
            "body": "This is a PR review comment",
            "created_at": "2023-01-06T12:30:00Z",
            "updated_at": "2023-01-06T12:30:00Z",
            "html_url": "https://github.com/{}/{}/pull/3#discussion_r789012",
            "pull_request_url": "https://api.github.com/repos/{}/{}/pulls/3"
        }},
        {{
            "id": 345678,
            "node_id": "MDI0OlB1bGxSZXF1ZXN0UmV2aWV3Q29tbWVudDM0NTY3OA==",
            "pull_request_review_id": 67890,
            "diff_hunk": "@@ -10,5 +10,6 @@\\n   console.log('test');\\n+  // Another change\\n   return result;\\n }}",
            "path": "src/main.rs",
            "position": 2,
            "original_position": 2,
            "commit_id": "abcdef1234567890",
            "original_commit_id": "0987654321fedcba",
            "user": {{
                "login": "test-user2",
                "id": 67890
            }},
            "body": "Another PR review comment",
            "created_at": "2023-01-06T13:30:00Z",
            "updated_at": "2023-01-06T13:30:00Z",
            "html_url": "https://github.com/{}/{}/pull/3#discussion_r345678",
            "pull_request_url": "https://api.github.com/repos/{}/{}/pulls/3"
        }}
    ]"#, owner, repo, owner, repo, owner, repo, owner, repo);

    server.mock("GET", &format!("/repos/{}/{}/pulls/comments", owner, repo))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for GET /repos/{owner}/{repo}/commits/{commit_sha}/comments
pub fn mock_commit_comments(server: &Server, owner: &str, repo: &str, commit_sha: &str) -> Mock {
    let response_body = format!(r#"[
        {{
            "id": 901234,
            "node_id": "MDEzOkNvbW1pdENvbW1lbnQ5MDEyMzQ=",
            "url": "https://api.github.com/repos/{}/{}/comments/901234",
            "html_url": "https://github.com/{}/{}/commit/{}#commitcomment-901234",
            "user": {{
                "login": "test-user1",
                "id": 12345
            }},
            "position": 1,
            "line": 1,
            "path": "src/main.rs",
            "commit_id": "{}",
            "created_at": "2023-01-08T10:00:00Z",
            "updated_at": "2023-01-08T10:00:00Z",
            "body": "This is a commit comment"
        }},
        {{
            "id": 567890,
            "node_id": "MDEzOkNvbW1pdENvbW1lbnQ1Njc4OTA=",
            "url": "https://api.github.com/repos/{}/{}/comments/567890",
            "html_url": "https://github.com/{}/{}/commit/{}#commitcomment-567890",
            "user": {{
                "login": "test-user2",
                "id": 67890
            }},
            "position": 2,
            "line": 2,
            "path": "src/main.rs",
            "commit_id": "{}",
            "created_at": "2023-01-08T11:00:00Z",
            "updated_at": "2023-01-08T11:00:00Z",
            "body": "Another commit comment"
        }}
    ]"#, owner, repo, owner, repo, commit_sha, commit_sha, owner, repo, owner, repo, commit_sha, commit_sha);

    server.mock("GET", &format!("/repos/{}/{}/commits/{}/comments", owner, repo, commit_sha))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for a 404 Not Found response
pub fn mock_not_found(server: &Server, endpoint: &str) -> Mock {
    let response_body = r#"{
        "message": "Not Found",
        "documentation_url": "https://docs.github.com/rest"
    }"#;

    server.mock("GET", endpoint)
        .with_status(404)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for a 401 Unauthorized response
pub fn mock_unauthorized(server: &Server, endpoint: &str) -> Mock {
    let response_body = r#"{
        "message": "Bad credentials",
        "documentation_url": "https://docs.github.com/rest"
    }"#;

    server.mock("GET", endpoint)
        .with_status(401)
        .with_header("content-type", "application/json")
        .with_body(response_body)
        .create()
}

/// Creates a mock for a 403 Rate Limited response
pub fn mock_rate_limited(server: &Server, endpoint: &str) -> Mock {
    let response_body = r#"{
        "message": "API rate limit exceeded",
        "documentation_url": "https://docs.github.com/rest/overview/resources-in-the-rest-api#rate-limiting"
    }"#;

    server.mock("GET", endpoint)
        .with_status(403)
        .with_header("content-type", "application/json")
        .with_header("x-ratelimit-limit", "60")
        .with_header("x-ratelimit-remaining", "0")
        .with_header("x-ratelimit-reset", "1611264000")
        .with_body(response_body)
        .create()
}

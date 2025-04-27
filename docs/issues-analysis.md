[Issues-Analysis](issues-analysis.md)

# Issues Analysis

Fetch and analyze GitHub issues from repositories:

## Usage Example

```python
# Fetch all issues (all pages by default)
issues = await manager.fetch_issues(repo_urls)

# Limit to the first 2 pages of results (up to 200 issues per repo)
issues = await manager.fetch_issues(repo_urls, max_pages=2)

# Optionally specify state to fetch only certain issues
open_issues = await manager.fetch_issues(repo_urls, state="open")
closed_issues = await manager.fetch_issues(repo_urls, state="closed", max_pages=1)
```

- By default, all pages of issues are fetched (no limit).
- Use `max_pages` to limit the number of pages (each page contains up to 100 issues).

```python
# Fetch issue information (default: all states - open, closed)
issues = await manager.fetch_issues(repo_urls)

# Optionally specify state to fetch only certain issues
open_issues = await manager.fetch_issues(repo_urls, state="open")
closed_issues = await manager.fetch_issues(repo_urls, state="closed")

# Process issue data
for repo_url, repo_result in issues.items():
    if isinstance(repo_result, str):
        # This is an error message
        print(f"Error fetching issues for {repo_url}: {repo_result}")
        continue

    print(f"\nRepository: {repo_url}")

    # Separate issues from pull requests (GitHub API returns both under issues endpoint)
    actual_issues = [issue for issue in repo_result if not issue['is_pull_request']]
    pull_requests = [issue for issue in repo_result if issue['is_pull_request']]

    print(f"Found {len(actual_issues)} issues and {len(pull_requests)} pull requests")

    # Count by state
    open_count = sum(1 for issue in actual_issues if issue['state'] == 'open')
    closed_count = sum(1 for issue in actual_issues if issue['state'] == 'closed')

    print(f"Issues: Open: {open_count}, Closed: {closed_count}")

    # Find the most common labels
    label_counts = {}
    for issue in actual_issues:
        for label in issue['labels']:
            if label not in label_counts:
                label_counts[label] = 0
            label_counts[label] += 1

    if label_counts:
        print("\nMost common labels:")
        for label, count in sorted(label_counts.items(), key=lambda x: x[1], reverse=True)[:5]:
            print(f"  - {label}: {count} issues")

    # Show the most recent issues
    recent_issues = sorted(actual_issues, key=lambda i: i['updated_at'], reverse=True)
    print("\nMost recently updated issues:")
    for issue in recent_issues[:5]:
        print(f"  - #{issue['number']} {issue['title']} ({issue['state']})")
        print(f"    Updated: {issue['updated_at']}")
        print(f"    Author: {issue['user_login']}")
        print(f"    Comments: {issue['comments_count']}")

# Convert to pandas DataFrame for analysis
import pandas as pd
from datetime import datetime

all_issues = []
for repo_url, repo_result in issues.items():
    if isinstance(repo_result, str):
        continue

    repo_name = '/'.join(repo_url.split('/')[-2:])

    # Process only actual issues (not PRs)
    actual_issues = [issue for issue in repo_result if not issue['is_pull_request']]

    for issue in actual_issues:
        # Convert string dates to datetime objects
        created_at = datetime.fromisoformat(issue['created_at'].replace('Z', '+00:00'))
        updated_at = datetime.fromisoformat(issue['updated_at'].replace('Z', '+00:00'))

        closed_at = None
        if issue['closed_at']:
            closed_at = datetime.fromisoformat(issue['closed_at'].replace('Z', '+00:00'))

        # Extract common properties for analysis
        issue_data = {
            'Repository': repo_name,
            'Number': issue['number'],
            'Title': issue['title'],
            'State': issue['state'],
            'Author': issue['user_login'],
            'Created': created_at,
            'Updated': updated_at,
            'Closed': closed_at,
            'Comments': issue['comments_count'],
            'Labels': ', '.join(issue['labels']) if issue['labels'] else '',
            'Assignees': ', '.join(issue['assignees']) if issue['assignees'] else '',
            'Milestone': issue['milestone'] if issue['milestone'] else '',
        }
        all_issues.append(issue_data)

# Create DataFrame
if all_issues:
    df = pd.DataFrame(all_issues)

    # Example analysis: Issue resolution time
    df['Created Date'] = pd.to_datetime(df['Created'])
    df['Closed Date'] = pd.to_datetime(df['Closed'])
    closed_issues = df[df['State'] == 'closed'].copy()
    if not closed_issues.empty:
        closed_issues['Days to Close'] = (closed_issues['Closed Date'] - closed_issues['Created Date']).dt.total_seconds() / (60*60*24)
        print("\nIssue Resolution Time:")
        print(f"Average Days to Close: {closed_issues['Days to Close'].mean():.2f}")
        print(f"Median Days to Close: {closed_issues['Days to Close'].median():.2f}")

    # Example analysis: Most productive issue resolvers
    if not closed_issues.empty and 'Assignees' in closed_issues.columns:
        # This is a simplified analysis since we only have comma-separated assignee names
        assignee_counts = {}
        for _, issue in closed_issues.iterrows():
            if issue['Assignees']:
                for assignee in issue['Assignees'].split(', '):
                    if assignee not in assignee_counts:
                        assignee_counts[assignee] = 0
                    assignee_counts[assignee] += 1

        if assignee_counts:
            print("\nMost Productive Issue Resolvers:")
            for assignee, count in sorted(assignee_counts.items(), key=lambda x: x[1], reverse=True)[:5]:
                print(f"  - {assignee}: {count} issues closed")

```
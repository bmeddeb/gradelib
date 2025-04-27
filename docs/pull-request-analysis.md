[Pull-Request-Analysis](pull-request-analysis.md)

# Pull Request Analysis

Fetch and analyze pull requests from repositories:

## Usage Example

```python
pull_requests = await manager.fetch_pull_requests(repo_urls)
# pull_requests is a dict: {repo_url: [list of pull requests] or error string}
# For each repo_url, the value is either a list of pull request dicts (on success)
# or an error string (on failure for that repo).
# No exceptions are raised for individual failures.
```

Fetch and analyze pull requests from repositories:

```python
# Fetch all pull requests (all pages by default)
pull_requests = await manager.fetch_pull_requests(repo_urls)

# Limit to the first 2 pages of results (up to 200 PRs per repo)
pull_requests = await manager.fetch_pull_requests(repo_urls, max_pages=2)

# Optionally specify state to fetch only certain pull requests
open_prs = await manager.fetch_pull_requests(repo_urls, state="open")
closed_prs = await manager.fetch_pull_requests(repo_urls, state="closed", max_pages=1)

# Process pull request data
for repo_url, repo_prs in pull_requests.items():
    if isinstance(repo_prs, str):
        # This is an error message
        print(f"Error fetching pull requests for {repo_url}: {repo_prs}")
        continue

    print(f"\nRepository: {repo_url}")
    print(f"Found {len(repo_prs)} pull requests")

    # Count by state
    open_count = sum(1 for pr in repo_prs if pr['state'] == 'open')
    closed_count = sum(1 for pr in repo_prs if pr['state'] == 'closed')
    merged_count = sum(1 for pr in repo_prs if pr['merged'])
    draft_count = sum(1 for pr in repo_prs if pr['is_draft'])

    print(f"Open: {open_count}, Closed: {closed_count}, Merged: {merged_count}, Draft: {draft_count}")

    # Find the most active PR authors
    authors = {}
    for pr in repo_prs:
        author = pr['user_login']
        if author not in authors:
            authors[author] = 0
        authors[author] += 1

    print("\nMost active PR authors:")
    for author, count in sorted(authors.items(), key=lambda x: x[1], reverse=True)[:5]:
        print(f"  - {author}: {count} PRs")

    # Show the most recent PRs
    recent_prs = sorted(repo_prs, key=lambda pr: pr['updated_at'], reverse=True)
    print("\nMost recently updated PRs:")
    for pr in recent_prs[:5]:
        print(f"  - #{pr['number']} {pr['title']} ({pr['state']})")
        print(f"    Updated: {pr['updated_at']}")
        print(f"    Author: {pr['user_login']}")
        print(f"    Changes: +{pr['additions']} -{pr['deletions']} in {pr['changed_files']} files")

# Fetch code review information (all pages by default)
code_reviews = await manager.fetch_code_reviews(repo_urls)

# Limit to the first 2 pages of pull requests (up to 200 PRs per repo)
code_reviews = await manager.fetch_code_reviews(repo_urls, max_pages=2)

- By default, all pages of pull requests are fetched for code review analysis (no limit).
- Use `max_pages` to limit the number of PR pages (each page contains up to 100 PRs).

# Convert to pandas DataFrame for analysis
import pandas as pd

all_prs = []
for repo_url, repo_prs in pull_requests.items():
    if isinstance(repo_prs, str):
        continue

    repo_name = '/'.join(repo_url.split('/')[-2:])

    for pr in repo_prs:
        # Extract common properties for analysis
        pr_data = {
            'Repository': repo_name,
            'Number': pr['number'],
            'Title': pr['title'],
            'State': pr['state'],
            'Author': pr['user_login'],
            'Created': pr['created_at'],
            'Updated': pr['updated_at'],
            'Closed': pr['closed_at'],
            'Merged': pr['merged_at'],
            'Is Merged': pr['merged'],
            'Comments': pr['comments'],
            'Commits': pr['commits'],
            'Additions': pr['additions'],
            'Deletions': pr['deletions'],
            'Changed Files': pr['changed_files'],
            'Is Draft': pr['is_draft'],
            'Labels': ', '.join(pr['labels'])
        }
        all_prs.append(pr_data)

# Create DataFrame
if all_prs:
    df = pd.DataFrame(all_prs)

    # Example analysis: PR size distribution
    df['Total Changes'] = df['Additions'] + df['Deletions']
    size_bins = [0, 10, 50, 100, 500, 1000, float('inf')]
    size_labels = ['XS (0-10)', 'S (11-50)', 'M (51-100)', 'L (101-500)', 'XL (501-1000)', 'XXL (1000+)']
    df['Size'] = pd.cut(df['Total Changes'], bins=size_bins, labels=size_labels)

    print("\nPR Size Distribution:")
    print(df['Size'].value_counts())

    # Example analysis: Average time to merge
    df['Created Date'] = pd.to_datetime(df['Created'])
    df['Merged Date'] = pd.to_datetime(df['Merged'])
    merged_prs = df[df['Is Merged'] == True].copy()
    if not merged_prs.empty:
        merged_prs['Days to Merge'] = (merged_prs['Merged Date'] - merged_prs['Created Date']).dt.total_seconds() / (60*60*24)
        print("\nAverage Days to Merge:", merged_prs['Days to Merge'].mean())
        print("Median Days to Merge:", merged_prs['Days to Merge'].median())

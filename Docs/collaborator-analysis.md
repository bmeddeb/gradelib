
[Collaborator-Analysis](collaborator-analysis.md)

# Collaborator Analysis

Fetch and analyze collaborators information for repositories:

```python
# Fetch collaborator information
collaborators = await manager.fetch_collaborators(repo_urls)

# Process collaborator data
for repo_url, repo_collaborators in collaborators.items():
    print(f"\nRepository: {repo_url}")
    print(f"Found {len(repo_collaborators)} collaborators")

    # Print collaborator information
    for collab in repo_collaborators:
        print(f"  - {collab['login']}")

        # Display additional information if available
        if collab.get('full_name'):
            print(f"    Name: {collab['full_name']}")

        if collab.get('email'):
            print(f"    Email: {collab['email']}")

# Convert to pandas DataFrame for analysis
import pandas as pd

all_collaborators = []
for repo_url, repo_collaborators in collaborators.items():
    repo_name = '/'.join(repo_url.split('/')[-2:])

    for collab in repo_collaborators:
        collab_data = {
            'Repository': repo_name,
            'Login': collab['login'],
            'GitHub ID': collab['github_id'],
            'Name': collab.get('full_name', 'N/A'),
            'Email': collab.get('email', 'N/A'),
        }
        all_collaborators.append(collab_data)

# Create DataFrame
df = pd.DataFrame(all_collaborators)
print("\nCollaborator DataFrame:")
print(df)

```
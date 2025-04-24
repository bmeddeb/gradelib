import pytest
import gradelib

# Public Taiga project base and slug
PUBLIC_TAIGA_BASE_URL = "https://api.taiga.io/api/v1/"
PUBLIC_TAIGA_SLUG = "ibarraz5-ser402-team3"  # Use a valid public project

@pytest.fixture(scope="module")
def public_taiga_client():
    return gradelib.TaigaClient(
        base_url=PUBLIC_TAIGA_BASE_URL,
        auth_token="",  # No token for public access
        username=""
    )

@pytest.mark.asyncio
async def test_fetch_project_data(public_taiga_client):
    """Fetch complete data for a public project (no auth required)."""
    result = await public_taiga_client.fetch_project_data(PUBLIC_TAIGA_SLUG)
    assert "project" in result
    assert "members" in result
    assert "sprints" in result
    assert "user_stories" in result
    assert "tasks" in result
    assert "task_histories" in result

@pytest.mark.asyncio
async def test_fetch_multiple_projects(public_taiga_client):
    """Fetch the same project multiple times via bulk API."""
    result = await public_taiga_client.fetch_multiple_projects([
        PUBLIC_TAIGA_SLUG,
        PUBLIC_TAIGA_SLUG
    ])
    assert isinstance(result, dict)
    assert PUBLIC_TAIGA_SLUG in result
    assert result[PUBLIC_TAIGA_SLUG] is True or "Error" in result[PUBLIC_TAIGA_SLUG]

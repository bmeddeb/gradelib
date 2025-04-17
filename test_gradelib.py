import asyncio
import gradelib


def main():
    # Initialize the async runtime
    gradelib.setup_async()

    # Create a GitHub provider
    provider = gradelib.GitHubProvider(
        username="test_user",
        token="test_token",
        urls=["https://github.com/octocat/Hello-World"]
    )

    # Print the provider
    print(f"Provider: {provider}")

    # Print methods
    print(f"Methods: {dir(provider)}")


if __name__ == "__main__":
    main()

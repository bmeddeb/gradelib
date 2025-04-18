#!/usr/bin/env python3
"""
Example script demonstrating the fetch_code_reviews functionality in gradelib.

This script:
1. Sets up the async environment
2. Creates a RepoManager with GitHub credentials
3. Fetches code review information for specified repositories
4. Displays the code review data in a DataFrame
5. Generates visualizations of code review patterns
"""

import asyncio
import os
import sys
import pandas as pd
import matplotlib.pyplot as plt
import seaborn as sns
from datetime import datetime, timedelta
from typing import Dict, List
from collections import defaultdict

# Add the parent directory to path if running the script directly
if __name__ == "__main__":
    sys.path.insert(0, os.path.abspath(os.path.join(os.path.dirname(__file__), '..')))

# Import gradelib
from gradelib.gradelib import setup_async, RepoManager


async def main():
    # Initialize the async runtime environment
    setup_async()
    
    # Get GitHub credentials from environment variables (for security)
    github_username = os.environ.get("GITHUB_USERNAME", "")
    github_token = os.environ.get("GITHUB_TOKEN", "")
    
    if not github_username or not github_token:
        print("Error: GITHUB_USERNAME and GITHUB_TOKEN environment variables must be set")
        print("Example: export GITHUB_USERNAME=yourusername")
        print("         export GITHUB_TOKEN=your_personal_access_token")
        sys.exit(1)
    
    # Define the repositories to analyze
    repo_urls = [
        "https://github.com/bmeddeb/gradelib",
        "https://github.com/PyO3/pyo3",
        "https://github.com/bmeddeb/SER402-Team3",
    ]
    
    # Create the repo manager
    manager = RepoManager(repo_urls, github_username, github_token)
    
    # Fetch code review information
    print(f"Fetching code review information for {len(repo_urls)} repositories...")
    try:
        code_reviews = await manager.fetch_code_reviews(repo_urls)
        
        # Process and display the code review data
        all_reviews = []
        total_review_count = 0
        total_pr_count = 0
        
        # Store review timelines for visualization
        review_timeline = defaultdict(int)
        review_states_by_repo = defaultdict(lambda: defaultdict(int))
        reviewer_activity = defaultdict(int)
        
        for repo_url, repo_result in code_reviews.items():
            repo_name = '/'.join(repo_url.split('/')[-2:])
            
            if isinstance(repo_result, str):
                # This is an error message
                print(f"\nError for repository {repo_name}: {repo_result}")
                continue
                
            print(f"\nRepository: {repo_name}")
            
            # Count PRs with reviews and total reviews
            pr_count = len(repo_result)
            review_count = sum(len(reviews) for reviews in repo_result.values())
            
            print(f"Found {review_count} reviews for {pr_count} pull requests")
            total_review_count += review_count
            total_pr_count += pr_count
            
            # Process the reviews
            for pr_number, reviews in repo_result.items():
                for review in reviews:
                    # Convert dates to datetime objects for better display
                    submitted_at = datetime.fromisoformat(review['submitted_at'].replace('Z', '+00:00'))
                    
                    # Update timeline data (by day)
                    review_day = submitted_at.date()
                    review_timeline[review_day] += 1
                    
                    # Update reviewer activity
                    reviewer_activity[review['user_login']] += 1
                    
                    # Update state counts by repo
                    review_states_by_repo[repo_name][review['state']] += 1
                    
                    # Add repository information to each review record
                    review_data = {
                        'Repository': repo_name,
                        'PR Number': review['pr_number'],
                        'Review ID': review['id'],
                        'Reviewer': review['user_login'],
                        'State': review['state'],
                        'Submitted At': submitted_at,
                        'Commit ID': review['commit_id'][:8],  # Shortened commit ID
                        'Body': review['body'][:50] + '...' if review['body'] and len(review['body']) > 50 else review['body'],
                        'URL': review['html_url'],
                        'Review Day': review_day,
                        'Review Week': review_day - timedelta(days=review_day.weekday()),  # Start of the week
                        'Review Month': review_day.replace(day=1),  # Start of the month
                    }
                    
                    all_reviews.append(review_data)
        
        # Create a DataFrame from all review records
        if all_reviews:
            df = pd.DataFrame(all_reviews)
            
            # Display the DataFrame
            print(f"\nTotal Reviews: {total_review_count}")
            print(f"Total PRs with Reviews: {total_pr_count}")
            pd.set_option('display.max_columns', None)
            pd.set_option('display.width', 1000)
            pd.set_option('display.max_colwidth', 50)  # Truncate long text
            
            # Sort by submitted date (newest first)
            df = df.sort_values('Submitted At', ascending=False)
            
            print("\nMost Recent Reviews:")
            print(df.head(10))  # Show top 10 most recent reviews
            
            # Some basic statistics
            print("\nReview Statistics:")
            if 'State' in df.columns:
                # Review state counts
                state_counts = df['State'].value_counts()
                print("\nReview States:")
                print(state_counts)
                
                if 'APPROVED' in state_counts:
                    print(f"\nApproval Rate: {state_counts.get('APPROVED', 0) / len(df):.2%}")
                
                if 'CHANGES_REQUESTED' in state_counts:
                    print(f"Changes Requested Rate: {state_counts.get('CHANGES_REQUESTED', 0) / len(df):.2%}")
            
            # Repository statistics
            print("\nReviews by Repository:")
            print(df['Repository'].value_counts())
            
            # Reviewer statistics
            print("\nTop Reviewers:")
            print(df['Reviewer'].value_counts().head(10))  # Top 10 reviewers
            
            # PRs with most reviews
            print("\nPRs with Most Reviews:")
            pr_review_counts = df.groupby(['Repository', 'PR Number']).size().sort_values(ascending=False)
            print(pr_review_counts.head(10))  # Top 10 most reviewed PRs
            
            # Average review time statistics (if we had PR creation times)
            
            # Time-based analysis
            print("\nReview Activity by Day of Week:")
            day_of_week_counts = df['Submitted At'].dt.day_name().value_counts()
            print(day_of_week_counts)
            
            print("\nReview Activity by Hour of Day:")
            hour_counts = df['Submitted At'].dt.hour.value_counts().sort_index()
            print(hour_counts)
            
            # Save to CSV
            csv_path = os.path.join(os.path.dirname(__file__), 'code_review_analysis.csv')
            df.to_csv(csv_path, index=False)
            print(f"\nCode review data saved to: {csv_path}")
            
            # Generate visualizations
            try:
                output_dir = os.path.join(os.path.dirname(__file__), 'visualizations')
                os.makedirs(output_dir, exist_ok=True)
                
                print("\nGenerating visualizations...")
                
                # Set up the style
                plt.style.use('ggplot')
                
                # 1. Review timeline
                plt.figure(figsize=(12, 6))
                # Convert defaultdict to Series and sort by date
                timeline_series = pd.Series(dict(review_timeline)).sort_index()
                # Fill in missing dates with zeros
                if len(timeline_series) > 1:
                    date_range = pd.date_range(start=timeline_series.index.min(), end=timeline_series.index.max())
                    timeline_series = timeline_series.reindex(date_range, fill_value=0)
                timeline_series.plot(marker='o', linestyle='-')
                plt.title('Code Review Activity Over Time')
                plt.xlabel('Date')
                plt.ylabel('Number of Reviews')
                plt.tight_layout()
                plt.savefig(os.path.join(output_dir, 'review_timeline.png'))
                
                # 2. Review states by repository
                if review_states_by_repo:
                    plt.figure(figsize=(12, 8))
                    states_df = pd.DataFrame(review_states_by_repo).fillna(0)
                    states_df.plot(kind='bar', stacked=True)
                    plt.title('Review States by Repository')
                    plt.xlabel('Review State')
                    plt.ylabel('Count')
                    plt.legend(title='Repository')
                    plt.tight_layout()
                    plt.savefig(os.path.join(output_dir, 'review_states_by_repo.png'))
                
                # 3. Top reviewers pie chart
                if reviewer_activity:
                    plt.figure(figsize=(10, 10))
                    # Get top 10 reviewers
                    top_reviewers = dict(sorted(reviewer_activity.items(), key=lambda x: x[1], reverse=True)[:10])
                    plt.pie(top_reviewers.values(), labels=top_reviewers.keys(), autopct='%1.1f%%', 
                            shadow=True, startangle=90)
                    plt.axis('equal')
                    plt.title('Top 10 Reviewers')
                    plt.tight_layout()
                    plt.savefig(os.path.join(output_dir, 'top_reviewers.png'))
                
                # 4. Day of week activity
                plt.figure(figsize=(10, 6))
                day_order = ['Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday']
                day_of_week_counts = day_of_week_counts.reindex(day_order)
                day_of_week_counts.plot(kind='bar')
                plt.title('Review Activity by Day of Week')
                plt.xlabel('Day of Week')
                plt.ylabel('Number of Reviews')
                plt.tight_layout()
                plt.savefig(os.path.join(output_dir, 'day_of_week_activity.png'))
                
                # 5. Hour of day activity
                plt.figure(figsize=(12, 6))
                hour_counts.plot(kind='bar')
                plt.title('Review Activity by Hour of Day')
                plt.xlabel('Hour (UTC)')
                plt.ylabel('Number of Reviews')
                plt.xticks(rotation=0)
                plt.tight_layout()
                plt.savefig(os.path.join(output_dir, 'hour_activity.png'))
                
                print(f"Visualizations saved to: {output_dir}")
                
            except Exception as viz_error:
                print(f"Error generating visualizations: {viz_error}")
                import traceback
                traceback.print_exc()
                
        else:
            print("\nNo code review data found.")
            
    except Exception as e:
        print(f"Error fetching code reviews: {e}")
        import traceback
        traceback.print_exc()


if __name__ == "__main__":
    asyncio.run(main()) 
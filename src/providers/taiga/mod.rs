// Taiga provider modules

// Common types and utilities for Taiga API interactions
pub(crate) mod common;

// API client for Taiga
pub(crate) mod client;

// Module for handling Taiga projects
pub(crate) mod projects;

// Module for handling Taiga sprints/milestones
pub(crate) mod sprints;

// Module for handling Taiga user stories
pub(crate) mod user_stories;

// Module for handling Taiga tasks
pub(crate) mod tasks;

// Module for handling Taiga issues
pub(crate) mod issues;

// Module for handling Taiga task history
pub(crate) mod task_history;

// Orchestrator for coordinating Taiga API operations
pub(crate) mod orchestrator;

// Re-export common items

#!/usr/bin/env nu
################################################################################
# Script Name: github.nu
# Description: GitHub automation utilities for DGX-Pixels project
# Author: dgx-pixels project
# Created: 2025-11-10
# Modified: 2025-11-10
#
# Usage: use scripts/nu/modules/github.nu *
#
# Provides:
#   - gh-create-branch: Create branch for workstream
#   - gh-create-pr: Create PR with template
#   - gh-auto-merge: Enable auto-merge after CI passes
#   - gh-rebase-main: Rebase current branch onto main
#   - gh-check-status: Check CI status for current branch
#   - gh-list-prs: List open pull requests
#
# Dependencies:
#   - nushell >= 0.96
#   - gh (GitHub CLI) >= 2.0
#   - git >= 2.0
################################################################################

use ../config.nu [COLORS, log-success, log-error, log-warning, log-info, command-exists, current-branch, is-git-clean]

# Create a new branch for a workstream
#
# Creates and checks out a new git branch with standardized naming
#
# Parameters:
#   branch_name: string - Name for the new branch (will be kebab-cased)
#   base_branch?: string - Branch to create from (default: "main")
#
# Returns: bool - true if branch was created successfully
#
# Example:
#   gh-create-branch "implement-comfyui-integration"
#   gh-create-branch "fix-memory-leak" "develop"
export def gh-create-branch [
    branch_name: string,
    base_branch: string = "main"
] {
    if not (command-exists "git") {
        log-error "git not found - please install git"
        return false
    }

    # Ensure we're on the base branch and it's up to date
    try {
        log-info $"Switching to base branch: ($base_branch)"
        git checkout $base_branch

        log-info "Pulling latest changes..."
        git pull origin $base_branch

        # Create and checkout new branch
        log-info $"Creating new branch: ($branch_name)"
        git checkout -b $branch_name

        log-success $"Created and checked out branch: ($branch_name)"
        return true

    } catch {|err|
        log-error $"Failed to create branch: ($err.msg)"
        return false
    }
}

# Create a pull request with template
#
# Creates a GitHub PR using gh CLI with optional template and auto-fill
#
# Parameters:
#   title: string - PR title
#   body?: string - PR description (optional, will use template if not provided)
#   base?: string - Base branch for PR (default: "main")
#   draft?: bool - Create as draft PR (default: false)
#   labels?: list<string> - Labels to add to PR (default: [])
#
# Returns: record - PR information including URL and number
#
# Example:
#   gh-create-pr "Add ComfyUI integration" --draft
#   gh-create-pr "Fix memory leak" --base develop --labels [bug, critical]
export def gh-create-pr [
    title: string,
    --body: string,
    --base: string = "main",
    --draft,
    --labels: list<string> = []
] {
    if not (command-exists "gh") {
        log-error "gh CLI not found - please install GitHub CLI"
        log-info "Install: https://cli.github.com/"
        return {
            success: false
            url: null
            number: null
            error: "gh CLI not found"
        }
    }

    let current = (current-branch)

    if $current == $base {
        log-error "Cannot create PR from the same branch as base"
        return {
            success: false
            url: null
            number: null
            error: "Current branch is same as base branch"
        }
    }

    try {
        # Build gh pr create command
        mut cmd_args = ["pr", "create", "--base", $base, "--title", $title]

        # Add body if provided, otherwise use template
        if ($body != null) and ($body != "") {
            $cmd_args = ($cmd_args | append ["--body", $body])
        } else {
            # Check if PR template exists
            let template_paths = [
                ".github/PULL_REQUEST_TEMPLATE.md",
                ".github/pull_request_template.md",
                "docs/PULL_REQUEST_TEMPLATE.md"
            ]

            let template = ($template_paths | where {|path| $path | path exists} | first)

            if ($template != null) {
                $cmd_args = ($cmd_args | append ["--body-file", $template])
                log-info $"Using PR template: ($template)"
            } else {
                $cmd_args = ($cmd_args | append ["--fill"])
                log-info "Using auto-fill for PR body"
            }
        }

        # Add draft flag
        if $draft {
            $cmd_args = ($cmd_args | append "--draft")
            log-info "Creating as draft PR"
        }

        # Add labels
        if ($labels | length) > 0 {
            let labels_str = ($labels | str join ",")
            $cmd_args = ($cmd_args | append ["--label", $labels_str])
            log-info $"Adding labels: ($labels_str)"
        }

        # Create PR
        log-info $"Creating PR from ($current) to ($base)..."
        let pr_url = (gh ...$cmd_args)

        # Extract PR number from URL
        let pr_number = ($pr_url | split row "/" | last | str trim)

        log-success $"PR created: ($pr_url)"

        return {
            success: true
            url: $pr_url
            number: ($pr_number | into int)
            error: null
        }

    } catch {|err|
        log-error $"Failed to create PR: ($err.msg)"
        return {
            success: false
            url: null
            number: null
            error: $err.msg
        }
    }
}

# Enable auto-merge for a pull request
#
# Enables GitHub's auto-merge feature for a PR after CI passes
#
# Parameters:
#   pr_number?: int - PR number (default: current branch's PR)
#   merge_method?: string - Merge method: merge, squash, rebase (default: "squash")
#
# Returns: bool - true if auto-merge was enabled
#
# Example:
#   gh-auto-merge
#   gh-auto-merge 42 --merge-method rebase
export def gh-auto-merge [
    pr_number?: int,
    --merge-method: string = "squash"
] {
    if not (command-exists "gh") {
        log-error "gh CLI not found"
        return false
    }

    # Validate merge method
    if not ($merge_method in ["merge", "squash", "rebase"]) {
        log-error $"Invalid merge method: ($merge_method). Must be: merge, squash, or rebase"
        return false
    }

    try {
        # If PR number not provided, get it from current branch
        let pr_num = if ($pr_number != null) {
            $pr_number
        } else {
            let pr_list = (gh pr list --head (current-branch) --json number | from json)

            if ($pr_list | length) == 0 {
                log-error "No PR found for current branch"
                return false
            }

            ($pr_list | first | get number)
        }

        log-info $"Enabling auto-merge for PR #($pr_num) with method: ($merge_method)"

        gh pr merge $pr_num --auto --($merge_method)

        log-success $"Auto-merge enabled for PR #($pr_num)"
        log-info "PR will be merged automatically when all checks pass"

        return true

    } catch {|err|
        log-error $"Failed to enable auto-merge: ($err.msg)"
        return false
    }
}

# Rebase current branch onto main
#
# Rebases the current branch onto the latest main branch
#
# Parameters:
#   base_branch?: string - Branch to rebase onto (default: "main")
#   interactive?: bool - Use interactive rebase (default: false)
#
# Returns: bool - true if rebase succeeded
#
# Example:
#   gh-rebase-main
#   gh-rebase-main --base develop --interactive
export def gh-rebase-main [
    --base: string = "main",
    --interactive
] {
    if not (command-exists "git") {
        log-error "git not found"
        return false
    }

    let current = (current-branch)

    if $current == $base {
        log-error $"Already on base branch: ($base)"
        return false
    }

    if not (is-git-clean) {
        log-error "Working directory has uncommitted changes"
        log-info "Please commit or stash your changes before rebasing"
        return false
    }

    try {
        # Fetch latest changes
        log-info "Fetching latest changes from origin..."
        git fetch origin

        # Rebase onto base branch
        if $interactive {
            log-info $"Starting interactive rebase onto origin/($base)..."
            log-warning "Interactive rebase will open an editor"
            git rebase -i $"origin/($base)"
        } else {
            log-info $"Rebasing onto origin/($base)..."
            git rebase $"origin/($base)"
        }

        log-success $"Successfully rebased ($current) onto ($base)"
        log-info "Push with: git push --force-with-lease"

        return true

    } catch {|err|
        log-error $"Rebase failed: ($err.msg)"
        log-info "Resolve conflicts and run: git rebase --continue"
        log-info "Or abort with: git rebase --abort"
        return false
    }
}

# Check CI status for current branch or PR
#
# Displays the status of GitHub Actions checks
#
# Parameters:
#   pr_number?: int - PR number (default: current branch's PR)
#
# Returns: record - CI status information
#
# Example:
#   let status = (gh-check-status)
#   if $status.all_passed {
#       print "All checks passed!"
#   }
export def gh-check-status [
    pr_number?: int
] {
    if not (command-exists "gh") {
        log-error "gh CLI not found"
        return {
            success: false
            all_passed: false
            checks: []
            error: "gh CLI not found"
        }
    }

    try {
        # If PR number not provided, get it from current branch
        let pr_num = if ($pr_number != null) {
            $pr_number
        } else {
            let pr_list = (gh pr list --head (current-branch) --json number | from json)

            if ($pr_list | length) == 0 {
                log-warning "No PR found for current branch - checking commit status"

                # Fall back to checking commit status
                let checks = (gh api repos/:owner/:repo/commits/(git rev-parse HEAD)/check-runs | from json)

                let check_runs = ($checks.check_runs | select name status conclusion)

                let all_passed = ($check_runs | all {|check| $check.conclusion == "success"})

                return {
                    success: true
                    all_passed: $all_passed
                    checks: $check_runs
                    error: null
                }
            }

            ($pr_list | first | get number)
        }

        log-info $"Checking CI status for PR #($pr_num)..."

        # Get PR checks
        let checks = (gh pr checks $pr_num --json name,status,conclusion | from json)

        # Display check status
        for check in $checks {
            let status_icon = if $check.conclusion == "success" {
                $"($COLORS.success)✓($COLORS.reset)"
            } else if $check.conclusion == "failure" {
                $"($COLORS.error)✗($COLORS.reset)"
            } else if $check.status == "in_progress" {
                $"($COLORS.warning)⋯($COLORS.reset)"
            } else {
                $"($COLORS.info)○($COLORS.reset)"
            }

            print $"($status_icon) ($check.name): ($check.status) ($check.conclusion)"
        }

        let all_passed = ($checks | all {|check| $check.conclusion == "success"})

        if $all_passed {
            log-success "All checks passed!"
        } else {
            let failed_count = ($checks | where conclusion == "failure" | length)
            let pending_count = ($checks | where status == "in_progress" | length)

            if $failed_count > 0 {
                log-error $"($failed_count) check(s) failed"
            }
            if $pending_count > 0 {
                log-info $"($pending_count) check(s) in progress"
            }
        }

        return {
            success: true
            all_passed: $all_passed
            checks: $checks
            error: null
        }

    } catch {|err|
        log-error $"Failed to check CI status: ($err.msg)"
        return {
            success: false
            all_passed: false
            checks: []
            error: $err.msg
        }
    }
}

# List open pull requests
#
# Lists all open PRs for the repository with filtering options
#
# Parameters:
#   --author: string - Filter by author
#   --label: string - Filter by label
#   --limit: int - Maximum number of PRs to show (default: 10)
#
# Returns: table - List of PRs with details
#
# Example:
#   gh-list-prs
#   gh-list-prs --author "@me" --limit 5
export def gh-list-prs [
    --author: string,
    --label: string,
    --limit: int = 10
] {
    if not (command-exists "gh") {
        log-error "gh CLI not found"
        return []
    }

    try {
        mut cmd_args = ["pr", "list", "--json", "number,title,author,createdAt,headRefName,state,isDraft", "--limit", ($limit | into string)]

        if ($author != null) {
            $cmd_args = ($cmd_args | append ["--author", $author])
        }

        if ($label != null) {
            $cmd_args = ($cmd_args | append ["--label", $label])
        }

        let prs = (gh ...$cmd_args | from json)

        if ($prs | length) == 0 {
            log-info "No open pull requests found"
            return []
        }

        let formatted = (
            $prs
            | select number title author.login headRefName state isDraft
            | rename pr_number title author branch state draft
        )

        log-info $"Found ($prs | length) open PR(s)"

        return $formatted

    } catch {|err|
        log-error $"Failed to list PRs: ($err.msg)"
        return []
    }
}

# Get PR details
#
# Retrieves detailed information about a specific PR
#
# Parameters:
#   pr_number: int - PR number to query
#
# Returns: record - Detailed PR information
#
# Example:
#   let pr = (gh-get-pr-details 42)
#   print $"PR: ($pr.title) by ($pr.author)"
export def gh-get-pr-details [
    pr_number: int
] {
    if not (command-exists "gh") {
        log-error "gh CLI not found"
        return {}
    }

    try {
        let pr = (gh pr view $pr_number --json number,title,body,author,state,headRefName,baseRefName,createdAt,updatedAt,mergeable,isDraft,labels | from json)

        log-success $"Retrieved PR #($pr_number): ($pr.title)"

        return $pr

    } catch {|err|
        log-error $"Failed to get PR details: ($err.msg)"
        return {}
    }
}

# Close a pull request
#
# Closes a PR without merging
#
# Parameters:
#   pr_number: int - PR number to close
#   --comment: string - Optional closing comment
#
# Returns: bool - true if PR was closed successfully
#
# Example:
#   gh-close-pr 42 --comment "Superseded by #43"
export def gh-close-pr [
    pr_number: int,
    --comment: string
] {
    if not (command-exists "gh") {
        log-error "gh CLI not found"
        return false
    }

    try {
        if ($comment != null) {
            gh pr close $pr_number --comment $comment
        } else {
            gh pr close $pr_number
        }

        log-success $"Closed PR #($pr_number)"
        return true

    } catch {|err|
        log-error $"Failed to close PR: ($err.msg)"
        return false
    }
}

# Request PR review
#
# Requests review from specified reviewers
#
# Parameters:
#   pr_number?: int - PR number (default: current branch's PR)
#   reviewers: list<string> - List of GitHub usernames to request review from
#
# Returns: bool - true if review request succeeded
#
# Example:
#   gh-request-review [alice, bob]
#   gh-request-review 42 [charlie]
export def gh-request-review [
    reviewers: list<string>,
    pr_number?: int
] {
    if not (command-exists "gh") {
        log-error "gh CLI not found"
        return false
    }

    if ($reviewers | length) == 0 {
        log-error "No reviewers specified"
        return false
    }

    try {
        let pr_num = if ($pr_number != null) {
            $pr_number
        } else {
            let pr_list = (gh pr list --head (current-branch) --json number | from json)

            if ($pr_list | length) == 0 {
                log-error "No PR found for current branch"
                return false
            }

            ($pr_list | first | get number)
        }

        let reviewers_str = ($reviewers | str join ",")

        gh pr edit $pr_num --add-reviewer $reviewers_str

        log-success $"Requested review from: ($reviewers_str)"
        return true

    } catch {|err|
        log-error $"Failed to request review: ($err.msg)"
        return false
    }
}

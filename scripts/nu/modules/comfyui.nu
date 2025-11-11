#!/usr/bin/env nu
################################################################################
# Script Name: comfyui.nu
# Description: ComfyUI API wrapper for DGX-Pixels project
# Author: dgx-pixels project
# Created: 2025-11-10
# Modified: 2025-11-10
#
# Usage: use scripts/nu/modules/comfyui.nu *
#
# Provides:
#   - comfyui-url: Get ComfyUI URL from env or default
#   - comfyui-health-check: Check if ComfyUI is responding
#   - comfyui-generate: Submit generation workflow
#   - comfyui-get-queue: Get current queue status
#   - comfyui-get-history: Get generation history
#   - comfyui-interrupt: Interrupt current generation
#
# Dependencies:
#   - nushell >= 0.96
#   - http (nushell builtin)
#   - ComfyUI server running
################################################################################

use ../config.nu [COLORS, log-success, log-error, log-warning, log-info]

# Get ComfyUI URL from environment or default
#
# Checks COMFYUI_URL env var, falls back to http://localhost:8188
#
# Returns: string - The ComfyUI base URL
#
# Example:
#   comfyui-url
#   # => "http://localhost:8188"
export def comfyui-url [] {
    if "COMFYUI_URL" in $env {
        $env.COMFYUI_URL
    } else {
        "http://localhost:8188"
    }
}

# Check if ComfyUI server is responding
#
# Performs a GET request to /system_stats endpoint to verify ComfyUI is running
#
# Returns: bool - true if server is healthy, false otherwise
#
# Example:
#   if (comfyui-health-check) {
#       print "ComfyUI is running"
#   }
export def comfyui-health-check [] {
    let base_url = (comfyui-url)

    try {
        let response = (http get $"($base_url)/system_stats")

        if ($response | is-not-empty) {
            log-success $"ComfyUI is responding at ($base_url)"
            return true
        } else {
            log-error $"ComfyUI returned empty response at ($base_url)"
            return false
        }
    } catch {
        log-error $"Failed to connect to ComfyUI at ($base_url)"
        log-info "Make sure ComfyUI is running (check dgx-spark-playbooks)"
        return false
    }
}

# Get current queue status from ComfyUI
#
# Retrieves the current generation queue including pending and running jobs
#
# Returns: record - Queue status with queue_running and queue_pending lists
#
# Example:
#   let queue = (comfyui-get-queue)
#   print $"Running: ($queue.queue_running | length), Pending: ($queue.queue_pending | length)"
export def comfyui-get-queue [] {
    let base_url = (comfyui-url)

    try {
        let response = (http get $"($base_url)/queue")

        log-info $"Queue - Running: ($response.queue_running | length), Pending: ($response.queue_pending | length)"
        return $response
    } catch {
        log-error "Failed to get queue status from ComfyUI"
        return {
            queue_running: []
            queue_pending: []
        }
    }
}

# Submit a generation workflow to ComfyUI
#
# Posts a workflow JSON to ComfyUI's /prompt endpoint
#
# Parameters:
#   workflow: record - The workflow definition (typically loaded from JSON)
#   client_id?: string - Optional client ID for tracking (default: "dgx-pixels")
#
# Returns: record - Response with prompt_id and execution number
#
# Example:
#   let workflow = (open workflows/sprite-generation.json)
#   let result = (comfyui-generate $workflow)
#   print $"Prompt ID: ($result.prompt_id)"
export def comfyui-generate [
    workflow: record,
    client_id: string = "dgx-pixels"
] {
    let base_url = (comfyui-url)

    try {
        let payload = {
            prompt: $workflow,
            client_id: $client_id
        }

        log-info $"Submitting workflow to ComfyUI at ($base_url)"
        let response = (http post $"($base_url)/prompt" $payload)

        if "prompt_id" in $response {
            log-success $"Workflow submitted successfully. Prompt ID: ($response.prompt_id)"
            return $response
        } else {
            log-error "ComfyUI accepted request but did not return prompt_id"
            return $response
        }
    } catch {|err|
        log-error $"Failed to submit workflow to ComfyUI: ($err.msg)"
        return {
            error: $err.msg
            prompt_id: null
        }
    }
}

# Get generation history from ComfyUI
#
# Retrieves the history of completed and failed generations
#
# Parameters:
#   max_items?: int - Maximum number of history items to return (default: 10)
#
# Returns: record - History data indexed by prompt_id
#
# Example:
#   let history = (comfyui-get-history 5)
#   $history | transpose | each {|item| print $"($item.column0): ($item.column1.status)"}
export def comfyui-get-history [
    max_items: int = 10
] {
    let base_url = (comfyui-url)

    try {
        let response = (http get $"($base_url)/history?max_items=($max_items)")

        log-info $"Retrieved ($response | transpose | length) history items"
        return $response
    } catch {
        log-error "Failed to get history from ComfyUI"
        return {}
    }
}

# Get system stats from ComfyUI
#
# Retrieves system resource usage including GPU memory, RAM, etc.
#
# Returns: record - System statistics
#
# Example:
#   let stats = (comfyui-get-system-stats)
#   print $"GPU Memory Used: ($stats.devices.0.vram_used / 1024 / 1024 / 1024) GB"
export def comfyui-get-system-stats [] {
    let base_url = (comfyui-url)

    try {
        let response = (http get $"($base_url)/system_stats")

        if "devices" in $response {
            log-info $"Retrieved system stats for ($response.devices | length) device(s)"
        }
        return $response
    } catch {
        log-error "Failed to get system stats from ComfyUI"
        return {}
    }
}

# Interrupt the current generation
#
# Sends a POST request to interrupt any running generation
#
# Returns: bool - true if interrupt was successful
#
# Example:
#   if (comfyui-interrupt) {
#       print "Generation interrupted"
#   }
export def comfyui-interrupt [] {
    let base_url = (comfyui-url)

    try {
        http post $"($base_url)/interrupt" {}
        log-success "Sent interrupt signal to ComfyUI"
        return true
    } catch {
        log-error "Failed to send interrupt to ComfyUI"
        return false
    }
}

# Clear the pending queue
#
# Removes all pending items from the generation queue
#
# Returns: bool - true if queue was cleared successfully
#
# Example:
#   comfyui-clear-queue
export def comfyui-clear-queue [] {
    let base_url = (comfyui-url)

    try {
        let payload = { clear: true }
        http post $"($base_url)/queue" $payload
        log-success "Cleared ComfyUI queue"
        return true
    } catch {
        log-error "Failed to clear ComfyUI queue"
        return false
    }
}

# Get available models from ComfyUI
#
# Retrieves list of available checkpoints, LoRAs, and other models
#
# Returns: record - Available models by type
#
# Example:
#   let models = (comfyui-get-models)
#   print $"Available checkpoints: ($models.checkpoints | length)"
export def comfyui-get-models [] {
    let base_url = (comfyui-url)

    try {
        let response = (http get $"($base_url)/object_info")

        # Extract model information from the object_info response
        # ComfyUI returns node definitions which include model lists
        log-info "Retrieved model information from ComfyUI"
        return $response
    } catch {
        log-error "Failed to get models from ComfyUI"
        return {}
    }
}

# Wait for a specific prompt to complete
#
# Polls ComfyUI queue until the specified prompt_id is no longer running
#
# Parameters:
#   prompt_id: string - The prompt ID to wait for
#   timeout_seconds?: int - Maximum time to wait (default: 300)
#   poll_interval?: int - Seconds between status checks (default: 2)
#
# Returns: bool - true if completed, false if timed out
#
# Example:
#   let result = (comfyui-generate $workflow)
#   if (comfyui-wait-for-completion $result.prompt_id) {
#       print "Generation complete!"
#   }
export def comfyui-wait-for-completion [
    prompt_id: string,
    timeout_seconds: int = 300,
    poll_interval: int = 2
] {
    log-info $"Waiting for prompt ($prompt_id) to complete (timeout: ($timeout_seconds)s)"

    let start_time = (date now)

    loop {
        let queue = (comfyui-get-queue)

        # Check if prompt_id is in running or pending queue
        let is_running = ($queue.queue_running | any {|item| $item.0 == $prompt_id})
        let is_pending = ($queue.queue_pending | any {|item| $item.0 == $prompt_id})

        if not ($is_running or $is_pending) {
            log-success $"Prompt ($prompt_id) completed"
            return true
        }

        # Check timeout
        let current_time = (date now)
        let elapsed_duration = ($current_time - $start_time)

        # Convert duration to seconds for comparison
        if ($elapsed_duration | into int) > ($timeout_seconds * 1_000_000_000) {
            log-error $"Timeout waiting for prompt ($prompt_id) after ($timeout_seconds)s"
            return false
        }

        # Sleep for poll_interval seconds
        if $poll_interval == 1 {
            sleep 1sec
        } else if $poll_interval == 2 {
            sleep 2sec
        } else if $poll_interval == 3 {
            sleep 3sec
        } else if $poll_interval == 5 {
            sleep 5sec
        } else {
            sleep 2sec  # default
        }
    }
}

# Validate workflow JSON structure
#
# Checks if a workflow has the required structure for ComfyUI
#
# Parameters:
#   workflow: record - The workflow to validate
#
# Returns: bool - true if workflow is valid
#
# Example:
#   let workflow = (open workflows/sprite-generation.json)
#   if (comfyui-validate-workflow $workflow) {
#       print "Workflow is valid"
#   }
export def comfyui-validate-workflow [
    workflow: record
] {
    # ComfyUI workflows should have node definitions
    if ($workflow | is-empty) {
        log-error "Workflow is empty"
        return false
    }

    # Check if workflow contains node definitions (keys should be numeric strings)
    let keys = ($workflow | columns)

    if ($keys | is-empty) {
        log-error "Workflow has no nodes"
        return false
    }

    # Validate that each node has required fields
    for key in $keys {
        let node = ($workflow | get $key)

        if not ("class_type" in ($node | columns)) {
            log-error $"Node ($key) missing 'class_type' field"
            return false
        }

        if not ("inputs" in ($node | columns)) {
            log-error $"Node ($key) missing 'inputs' field"
            return false
        }
    }

    let node_count = ($keys | length)
    log-success $"Workflow validated successfully \(($node_count) nodes\)"
    return true
}

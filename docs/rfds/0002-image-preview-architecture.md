# RFD 0002: Image Preview Architecture for DGX-Pixels TUI

**Status**: Proposed
**Author**: Claude (AI Assistant)
**Date**: 2025-11-14
**PR**: #31 (fix-generation-trigger)

## Context

The DGX-Pixels TUI needs to display generated images directly in the terminal for rapid iteration. Currently, the preview panel shows only text-based information (filename, size, completion status) with a placeholder for future image rendering.

### Problem Statement

Users need to see generated pixel art immediately after generation without leaving the terminal. The current text-based preview requires users to manually open files in external viewers, breaking the rapid iteration workflow.

### Requirements

1. **Immediate Feedback**: Display generated images within <1 second of completion
2. **Terminal Compatibility**: Work across different terminal emulators (kitty, WezTerm, iTerm2, xterm, etc.)
3. **Performance**: 60+ FPS TUI rendering while displaying images
4. **Integration**: Align with project's Bevy game engine integration goals
5. **Maintainability**: Use well-tested libraries, avoid custom Sixel encoding

## Options Analysis

### Option 1: ratatui-image Library

**Description**: Use the `ratatui-image` crate, a mature library specifically designed for rendering images in ratatui applications.

**Technical Details**:
- Supports multiple protocols: Sixel, Kitty, iTerm2, Unicode half-blocks
- Automatic terminal capability detection
- Handles font-size pixel mapping
- Prevents TUI from overwriting image areas (using ratatui 0.23.0+ cell skipping)
- Active maintenance (latest: 2025)

**Pros**:
- ✅ **Battle-tested**: Widely used in ratatui ecosystem
- ✅ **Protocol abstraction**: Automatically chooses best available protocol
- ✅ **Simple integration**: Drop-in widget, works with existing code
- ✅ **No manual encoding**: Handles Sixel/Kitty/iTerm2 internally
- ✅ **Fallback support**: Unicode half-blocks for unsupported terminals
- ✅ **Fast implementation**: ~1-2 hours to integrate

**Cons**:
- ❌ **Limited to terminal capabilities**: Quality depends on terminal emulator
- ❌ **No GPU acceleration**: CPU-based rendering only
- ❌ **Fixed to terminal window**: Can't show images outside TUI

**Implementation Effort**: **Low** (1-2 hours)

**Code Example**:
```rust
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

// In App struct
pub struct App {
    image_picker: Picker,
    current_image: Option<Box<dyn StatefulProtocol>>,
    // ...
}

// Render image
if let Some(ref mut image) = app.current_image {
    let image_widget = StatefulImage::new(None);
    f.render_stateful_widget(image_widget, area, image);
}
```

**Recommendation for this option**: ✅ **Ideal for MVP and current sprint**

---

### Option 2: Manual Sixel Integration (viuer)

**Description**: Use the `viuer` crate (already in dependencies) to manually encode and render Sixel images, bypassing ratatui's buffer.

**Technical Details**:
- Direct crossterm raw writes
- Manual cursor positioning
- Custom Sixel encoding
- Requires careful coordination with ratatui rendering

**Pros**:
- ✅ **Already in dependencies**: No new crate needed
- ✅ **Full control**: Can optimize Sixel encoding
- ✅ **Lightweight**: viuer is a small crate

**Cons**:
- ❌ **Complex integration**: Must coordinate with ratatui's double-buffer
- ❌ **Terminal-specific**: Only works with Sixel-capable terminals
- ❌ **Potential flickering**: Raw writes can conflict with ratatui
- ❌ **No fallback**: Breaks on terminals without Sixel
- ❌ **Higher maintenance**: Custom code vs battle-tested library

**Implementation Effort**: **Medium** (4-8 hours)

**Recommendation for this option**: ❌ **Not recommended** - ratatui-image does this better

---

### Option 3: External Image Viewer

**Description**: Open images in external viewer (kitty icat, feh, eog) when generation completes.

**Technical Details**:
- Use `std::process::Command` to launch viewer
- TUI remains for controls
- Images shown in separate window/overlay

**Pros**:
- ✅ **Simple implementation**: Just shell commands
- ✅ **High quality**: Native image viewer rendering
- ✅ **Works everywhere**: Terminal-agnostic

**Cons**:
- ❌ **Poor UX**: Breaks immersion, requires window management
- ❌ **Slow workflow**: Extra steps to view images
- ❌ **Platform-specific**: Different commands per OS
- ❌ **No integration**: Disconnected from TUI state

**Implementation Effort**: **Very Low** (30 minutes)

**Recommendation for this option**: ⚠️  **Only as fallback** for unsupported terminals

---

### Option 4: bevy_ratatui Integration

**Description**: Migrate from pure ratatui to bevy_ratatui, using Bevy's ECS and rendering pipeline.

**Technical Details**:
- Use `bevy_ratatui` crate (cxreiff/bevy_ratatui)
- Bevy ECS for state management
- Native Bevy image rendering (GPU-accelerated)
- Ratatui widgets rendered in Bevy window
- Optional: `bevy_ratatui_camera` for rendering Bevy scenes to terminal

**Pros**:
- ✅ **Project alignment**: DGX-Pixels targets Bevy developers
- ✅ **Superior rendering**: GPU-accelerated, high-quality image display
- ✅ **Powerful architecture**: Bevy ECS for complex features
- ✅ **Future-proof**: Enables advanced features (3D previews, animations)
- ✅ **MCP integration**: Natural fit with planned Bevy MCP integration
- ✅ **Dual-mode**: Can render to terminal OR native window

**Cons**:
- ❌ **Major refactor**: Complete architecture change
- ❌ **Heavier dependencies**: Bevy is large (~50MB binary)
- ❌ **Different paradigm**: ECS vs imperative code
- ❌ **Learning curve**: Team must learn Bevy if unfamiliar
- ❌ **Complexity**: Overengineered for current MVP needs

**Implementation Effort**: **Very High** (2-3 weeks)

**Recommendation for this option**: 🎯 **Long-term strategic direction**

**Code Example**:
```rust
use bevy::prelude::*;
use bevy_ratatui::RatatuiPlugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RatatuiPlugins::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (handle_input, update_ui))
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Load generated image
    commands.spawn(ImageBundle {
        image: asset_server.load("outputs/job-123.png").into(),
        ..default()
    });
}
```

---

### Option 5: Hybrid Approach (bevy_ratatui_camera)

**Description**: Keep ratatui TUI, but use `bevy_ratatui_camera` to render Bevy camera view to terminal as ASCII/Unicode art.

**Technical Details**:
- Bevy scene with 2D sprite
- Camera renders to texture
- Texture converted to terminal characters
- Rendered as ratatui widget

**Pros**:
- ✅ **Best of both worlds**: Bevy rendering + ratatui TUI
- ✅ **Gradual migration**: Can adopt Bevy incrementally
- ✅ **Unique visuals**: ASCII art representation
- ✅ **Works in all terminals**: No Sixel required

**Cons**:
- ❌ **Lower quality**: ASCII conversion loses detail
- ❌ **Complex setup**: Two rendering pipelines
- ❌ **Performance overhead**: Bevy + terminal rendering
- ❌ **Still experimental**: bevy_ratatui_camera is newer

**Implementation Effort**: **High** (1-2 weeks)

**Recommendation for this option**: ⚠️  **Interesting, but premature**

---

## Decision Matrix

| Criterion                  | ratatui-image | Manual Sixel | External Viewer | bevy_ratatui | Hybrid |
|----------------------------|---------------|--------------|-----------------|--------------|--------|
| **Ease of Implementation** | ⭐⭐⭐⭐⭐       | ⭐⭐           | ⭐⭐⭐⭐⭐         | ⭐            | ⭐⭐     |
| **Image Quality**          | ⭐⭐⭐⭐        | ⭐⭐⭐⭐        | ⭐⭐⭐⭐⭐         | ⭐⭐⭐⭐⭐      | ⭐⭐⭐    |
| **Terminal Compatibility** | ⭐⭐⭐⭐⭐       | ⭐⭐           | ⭐⭐⭐⭐⭐         | ⭐⭐⭐⭐       | ⭐⭐⭐⭐⭐  |
| **Performance**            | ⭐⭐⭐⭐        | ⭐⭐⭐         | ⭐⭐⭐⭐          | ⭐⭐⭐⭐⭐      | ⭐⭐⭐    |
| **UX/Immersion**           | ⭐⭐⭐⭐⭐       | ⭐⭐⭐⭐⭐       | ⭐⭐             | ⭐⭐⭐⭐⭐      | ⭐⭐⭐⭐   |
| **Future-Proof**           | ⭐⭐⭐         | ⭐⭐           | ⭐              | ⭐⭐⭐⭐⭐      | ⭐⭐⭐⭐   |
| **Maintenance Burden**     | ⭐⭐⭐⭐⭐       | ⭐⭐           | ⭐⭐⭐⭐⭐         | ⭐⭐⭐        | ⭐⭐     |
| **Alignment with Goals**   | ⭐⭐⭐         | ⭐⭐           | ⭐⭐             | ⭐⭐⭐⭐⭐      | ⭐⭐⭐⭐   |
| **Total**                  | **32/40**     | **19/40**    | **26/40**       | **35/40**    | **27/40** |

## Recommendation

### Phased Approach (Recommended)

Adopt a **three-phase strategy** that delivers immediate value while progressing toward the ideal architecture:

#### Phase 1: **MVP with ratatui-image** (Current Sprint)
- **Timeline**: 1-2 hours
- **Goal**: Working image preview in TUI
- **Implementation**:
  1. Add `ratatui-image = "1.0"` to Cargo.toml
  2. Initialize `Picker` in `App::new()`
  3. Load images on generation complete
  4. Render in preview panel
  5. Fallback to text info for unsupported terminals

**Acceptance Criteria**:
- ✅ Images display in Sixel-capable terminals (kitty, WezTerm, iTerm2)
- ✅ Graceful degradation to Unicode half-blocks
- ✅ No performance regression
- ✅ Works with existing TUI layout

#### Phase 2: **Enhanced UX** (Next Sprint)
- **Timeline**: 2-3 days
- **Goal**: Polish image preview experience
- **Implementation**:
  1. Add image zoom controls (Ctrl+[+/-])
  2. Implement gallery navigation (Left/Right arrows)
  3. Add external viewer fallback option
  4. Show image metadata (dimensions, model used, seed)
  5. Cache rendered Sixel data

**Acceptance Criteria**:
- ✅ Users can zoom and pan images
- ✅ Navigate gallery without re-rendering
- ✅ Quick toggle to external viewer (Ctrl+O)
- ✅ Metadata overlay available

#### Phase 3: **bevy_ratatui Migration** (M4-M5 Milestone)
- **Timeline**: 2-3 weeks
- **Goal**: Full Bevy integration
- **Implementation**:
  1. Spike: Evaluate `bevy_ratatui` with small prototype
  2. Create migration plan for existing TUI screens
  3. Implement Bevy ECS for app state
  4. Migrate preview to Bevy native rendering
  5. Integrate with Bevy MCP for hot-reloading assets
  6. Support dual-mode: terminal + native window

**Acceptance Criteria**:
- ✅ All TUI functionality preserved
- ✅ GPU-accelerated image rendering
- ✅ Bevy MCP integration working
- ✅ Can toggle between terminal and native window
- ✅ Foundation for 3D previews, animations

### Why This Approach?

1. **Incremental Value**: Users get working preview immediately
2. **Risk Mitigation**: Validate architecture before major refactor
3. **Learning Path**: Team learns Bevy gradually
4. **Alignment**: Natural progression toward project goals
5. **Pragmatic**: Delivers now, evolves strategically

## Implementation Details

### Phase 1: ratatui-image Integration

**Step 1: Add Dependency**

```toml
# rust/Cargo.toml
[dependencies]
ratatui-image = "1.0"
```

**Step 2: Initialize Picker**

```rust
// rust/src/app.rs
use ratatui_image::{picker::Picker, protocol::StatefulProtocol};

pub struct App {
    // ... existing fields

    /// Image picker for protocol detection
    pub image_picker: Picker,

    /// Current preview image state
    pub current_image_state: Option<Box<dyn StatefulProtocol>>,
}

impl App {
    pub fn new() -> Self {
        let mut picker = Picker::from_termios().unwrap();
        picker.guess_protocol();

        Self {
            // ... existing initialization
            image_picker: picker,
            current_image_state: None,
        }
    }
}
```

**Step 3: Load Image on Generation Complete**

```rust
// rust/src/main.rs - in job complete handler
ProgressUpdate::JobComplete {
    job_id,
    image_path,
    duration_s,
} => {
    let path = PathBuf::from(&image_path);

    // Load image for preview
    if let Ok(dyn_img) = image::open(&path) {
        app.current_image_state = Some(app.image_picker.new_protocol(
            dyn_img,
            area,  // Use preview area dimensions
            ratatui_image::Resize::Fit(None),
        ));
    }

    // ... existing code
}
```

**Step 4: Render Image Widget**

```rust
// rust/src/ui/screens/generation.rs
use ratatui_image::StatefulImage;

fn render_preview_info(f: &mut Frame, area: ratatui::layout::Rect, app: &App) {
    if let Some(ref mut image_state) = app.current_image_state {
        // Render image
        let image_widget = StatefulImage::new(None);
        f.render_stateful_widget(image_widget, area, image_state);
    } else {
        // Fallback to text preview
        // ... existing code
    }
}
```

**Step 5: Handle Unsupported Terminals**

```rust
// Detect protocol on startup
match app.image_picker.protocol_type() {
    ProtocolType::Halfblocks => {
        info!("Terminal supports Unicode half-blocks (fallback mode)");
    }
    ProtocolType::Sixel => {
        info!("Terminal supports Sixel graphics");
    }
    ProtocolType::Kitty => {
        info!("Terminal supports Kitty graphics protocol");
    }
    ProtocolType::Iterm2 => {
        info!("Terminal supports iTerm2 graphics protocol");
    }
}
```

### Testing Plan

**Manual Testing**:
1. Test on kitty (Sixel)
2. Test on WezTerm (Sixel)
3. Test on iTerm2 (iTerm2 protocol)
4. Test on standard xterm (Unicode fallback)
5. Test on SSH session (degraded mode)

**Performance Testing**:
1. Measure frame time with 1024x1024 image
2. Verify 60+ FPS maintained
3. Test with rapid generation (5 images/minute)
4. Monitor memory usage with multiple cached images

**Integration Testing**:
1. Verify preview updates on new generation
2. Test gallery navigation
3. Confirm log viewer still works
4. Check all keyboard shortcuts

## Alternatives Considered

### Why Not Just External Viewer?

External viewers break immersion and slow workflow. DGX-Pixels is for **rapid iteration**—users shouldn't context-switch to view results.

### Why Not Manual Sixel Now?

`ratatui-image` is battle-tested, handles multiple protocols, and includes fallbacks. Reinventing this is premature optimization.

### Why Not bevy_ratatui Immediately?

While bevy_ratatui is the strategic direction, it's a 2-3 week refactor. Phase 1 delivers value in 2 hours. Users benefit faster, and we validate the need before committing.

## Migration Path to bevy_ratatui

When ready for Phase 3, the migration will be straightforward:

1. **State Migration**: `App` struct → Bevy Resources + Components
2. **Screen Abstraction**: Keep ratatui widgets, render via bevy_ratatui
3. **Image Handling**: Replace ratatui-image with Bevy `Image` assets
4. **Event System**: Crossterm events → Bevy Input events
5. **Hot Reloading**: Integrate with Bevy asset server for MCP

The current ratatui code becomes Bevy systems/resources with minimal changes:

```rust
// Before (ratatui)
fn render_preview(f: &mut Frame, area: Rect, app: &App) { ... }

// After (bevy_ratatui)
fn render_preview(
    mut ratatui: EventWriter<DrawRatatuiEvent>,
    app: Res<AppState>,
) {
    ratatui.send(DrawRatatuiEvent(|f| {
        // Same rendering logic
    }));
}
```

## Success Metrics

**Phase 1 (MVP)**:
- ✅ Time-to-preview: <1 second after generation
- ✅ Terminal compatibility: 80%+ of users see images
- ✅ Performance: 60+ FPS maintained
- ✅ User satisfaction: Positive feedback on preview feature

**Phase 2 (Enhanced UX)**:
- ✅ Gallery navigation: <100ms per image switch
- ✅ Zoom responsiveness: Immediate visual feedback
- ✅ Cache hit rate: >90% for recent images

**Phase 3 (bevy_ratatui)**:
- ✅ Feature parity: All Phase 1-2 features preserved
- ✅ Performance improvement: GPU rendering reduces CPU usage
- ✅ MCP integration: Hot-reload assets from Bevy projects
- ✅ Foundation ready: Can add 3D previews, animations

## Open Questions

1. **Should we render on demand or pre-cache all gallery images?**
   - Proposal: Lazy load on first view, cache for session

2. **What's the max image size before we should downsample?**
   - Proposal: Downsample to terminal dimensions (avoid wasted encoding)

3. **Do we need image format conversion (PNG → optimized format)?**
   - Proposal: Not for MVP; revisit if performance issue

4. **Should Phase 3 support both terminal and native window simultaneously?**
   - Proposal: Yes—allow toggling with Ctrl+W (window mode)

## References

- **ratatui-image**: https://github.com/benjajaja/ratatui-image
- **bevy_ratatui**: https://github.com/cxreiff/bevy_ratatui
- **ratatui Sixel support**: https://github.com/ratatui/ratatui/releases/tag/v0.23.0
- **viuer crate**: https://crates.io/crates/viuer
- **DGX-Pixels Architecture**: docs/07-rust-python-architecture.md
- **Bevy Integration Plan**: docs/04-bevy-integration.md

## Conclusion

The **phased approach** balances immediate user value with long-term strategic goals:

- **Phase 1 (ratatui-image)**: Ships in current sprint, solves immediate pain
- **Phase 2 (UX polish)**: Refines experience based on user feedback
- **Phase 3 (bevy_ratatui)**: Aligns with project vision, enables advanced features

This approach minimizes risk, delivers incrementally, and keeps us moving toward the ideal architecture without premature over-engineering.

**Recommended Action**: Approve Phase 1 implementation for current PR #31.

---

**Status**: ✅ **Ready for Review**
**Next Steps**: Team review → Approval → Implement Phase 1

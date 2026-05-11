# Dashboard Testing Plan

This plan outlines the strategy for adding tests to the `ao` dashboard, ensuring that each tab functions correctly and the application state remains consistent across user interactions.

## 1. Testing Infrastructure

To test the dashboard effectively, we will implement a two-tier testing approach:

### A. Unit Tests for `App` State (`src/dashboard/app.rs`)
These tests will verify the core logic of the dashboard without requiring a real terminal or a live system.
- **Mocking `sysinfo::System`**: We will wrap `sysinfo` usage in a way that allows us to inject predictable data for tests.
- **State Transition Tests**: Verify `next_tab`, `prev_tab`, `on_up`, `on_down`, and selection logic.
- **Filtering & Sorting**: Verify that process list filtering and sorting work as expected.

### B. UI Rendering Tests (`src/dashboard/*.rs`)
Using `ratatui::backend::TestBackend`, we can render the UI to an internal buffer and assert on its content.
- **Visual Verification**: Check that headers, footers, and tab names are correctly rendered.
- **Component Tests**: Verify that each tab (Overview, Storage, etc.) renders its specific widgets when active.

---

## 2. Test Cases per Tab

### 🏠 Overview Tab
- **Test Case**: `test_overview_history_updates`
- **Verification**: Ensure that calling `on_tick` correctly updates `cpu_history`, `mem_history`, etc., and that these values stay within bounds (e.g., 0-100%).

### ⚙ Process Tab
- **Test Case**: `test_process_selection_and_details`
- **Verification**:
  - Verify that `on_down` increments `selected_index` and correctly updates `scroll_offset`.
  - Verify that `fetch_process_details` correctly populates `process_details` (using a mock `/proc` structure).
  - Verify that the process details popup is rendered when `process_details` is `Some`.

### 💽 Storage Tab
- **Test Case**: `test_storage_list_rendering`
- **Verification**: Ensure all detected disks are listed and the usage bar correctly reflects the percentage (e.g., 50% usage shows 10 filled blocks).

### 👤 User Tab
- **Test Case**: `test_user_session_highlighting`
- **Verification**: Verify that sessions marked as "still logged in" are styled with `Color::Green`.

### 🌐 Network Tab
- **Test Case**: `test_network_speed_calculation`
- **Verification**: Provide two sets of network data with a known time delta and verify that `network_speeds` correctly calculates the throughput in B/s.

### 🛠 Service Tab
- **Test Case**: `test_service_status_styles`
- **Verification**: Ensure "active" services are green, "failed" are red, and others are yellow in the UI rendering.

### 🐳 Virtualization Tab
- **Test Case**: `test_container_list_population`
- **Verification**: Verify that `containers` vector is correctly mapped to the table rows.

### 🌡 Sensors Tab
- **Test Case**: `test_sensor_threshold_alerts`
- **Verification**: Mock a sensor above its critical threshold and verify that the UI applies the `BOLD` and `RED` modifiers.

### 📈 Charts Tab
- **Test Case**: `test_charts_data_mapping`
- **Verification**: Ensure the last 60 seconds of history data are correctly passed to the `Sparkline` and `Chart` widgets.

---

## 3. Implementation Steps

1. **Refactor `App` for Testability**: Introduce a trait or a way to inject mock system data.
2. **Add `TestBackend` support**: Create a helper function in `tests/common/mod.rs` to set up a `Terminal` with `TestBackend`.
3. **Implement Unit Tests**: Add `#[cfg(test)] mod tests` at the bottom of `app.rs`.
4. **Implement UI Tests**: Create `tests/dashboard_ui_tests.rs` for rendering-related assertions.

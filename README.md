# Ink

Ink is a GTK4 Layer Shell framework that is scriptable in Lua. It allows you to create custom desktop widgets and applications using Lua scripts to define the UI structure, behavior, and styling.

# Disclaimer

This is still in an early phase. You may encounter several bugs. You are free to make an Issue or a PR.

## Features

- **GTK4 Layer Shell:** Create desktop widgets that integrate seamlessly with your desktop environment.
- **Lua Scripting:** Easily define your UI and application logic using the Lua scripting language.
- **Hot Reloading:** Automatically reload your UI when you make changes to your Lua scripts.
- **CSS Styling:** Customize the appearance of your widgets using CSS.
- **Extensible:** A growing set of built-in services for accessing system information, sending notifications, and more.

## Getting Started

### Installation

To build the project from source, you need to have Rust and Cargo installed, as well as the GTK4 development libraries.

```bash
cargo build --release
```

The compiled binary will be located at `target/release/ink`.

### Creating a Configuration File

To create a default configuration file, run the `init` command:

```bash
./target/release/ink init
```

This will create an `init.lua` file in `~/.config/ink/`.

### Running the Application

To run the application, you need to provide a Lua configuration file:

```bash
./target/release/ink path/to/your/init.lua
```

If you don't provide a file, Ink will look for a default configuration at `~/.config/ink/init.lua`.

## Lua Scripting API

Your Lua script should return a table that defines the UI. This can be a single window definition or an array of window definitions.

### Window Definition

A window definition is a table with the following properties:

- `type`: The type of the root widget. This should be a valid GTK widget type (e.g., `GtkApplicationWindow`, `GtkWindow`).
- `window_mode`: (Optional) Can be set to `"layer_shell"` to create a layer shell window.
- `layer`: (Optional) The layer to display the window on. Can be `"background"`, `"bottom"`, `"top"`, or `"overlay"`.
- `anchors`: (Optional) A table that specifies how the window should be anchored to the edges of the screen.
- `margins`: (Optional) A table that specifies the margins of the window.
- `auto_exclusive_zone`: (Optional) A boolean that indicates whether the window should reserve space on the screen.
- `keyboard_mode`: (Optional) The keyboard interactivity mode.
- `css`: (Optional) A string containing CSS to be applied to the window.
- `css_path`: (Optional) The path to a CSS file to be loaded.
- `properties`: (Optional) A table of properties to be set on the root widget.
- `children`: (Optional) An array of child widget definitions.
- `signals`: (Optional) A table of signal handlers for the root widget.
- `keymaps`: (Optional) A table of keymaps to be set on the root widget.
- `actions`: (Optional) A table of actions to be added to the application.
- `menu`: (Optional) A table that defines the application's menu bar.
- `id`: (Optional) A unique identifier for the window.
- `realize`: (Optional) A function to be called when the window is realized.

### Widget Definition

A widget definition is a table with the following properties:

- `type`: The type of the widget (e.g., `GtkBox`, `GtkLabel`, `GtkButton`).
- `id`: (Optional) A unique identifier for the widget.
- `properties`: (Optional) A table of properties to be set on the widget.
- `children`: (Optional) An array of child widget definitions.
- `signals`: (Optional) A table of signal handlers for the widget.

### Widget Methods

Widgets returned by `app.get_widget_by_id(id)` or `build_ui(config)` have the following methods available:

- `destroy()`: Destroys the widget.
- `get_ancestor(gtype)`: Gets the first ancestor of a given GType.
- `find_child(name)`: Finds a child widget by its name.
- `set_text(text)`: Sets the text of a label or editable widget.
- `insert_text(text)`: Inserts text at the current cursor position in an editable widget.
- `set_visible(visible)`: Sets the visibility of the widget.
- `is_visible()`: Gets the visibility of the widget.
- `add_class(class)`: Adds a CSS class to the widget.
- `remove_class(class)`: Removes a CSS class from the widget.
- `remove_children()`: Removes all children from the widget.
- `grab_focus()`: Grabs focus for the widget.
- `get_text()`: Gets the text of an editable widget.
- `get_value()`: Gets the value of a range or progress bar.
- `set_value(value)`: Sets the value of a range or progress bar.
- `set_range(min, max)`: Sets the range of a range widget.
- `set_increments(step, page)`: Sets the increments of a range widget.
- `get_parent()`: Gets the parent of the widget.
- `queue_draw()`: Queues a redraw for the widget.
- `get_vadjustment()`: Gets the vertical adjustment of a scrolled window.
- `add_controller_motion(on_enter, on_leave)`: Adds a motion controller to the widget.
- `add(child, props)`: Adds a child widget.
- `set_property(key, value)`: Sets a property on the widget.
- `get_property(key)`: Gets a property from the widget.
- `connect_signal(signal_name, func)`: Connects a signal handler to the widget.

### Globals

The following global functions and tables are available in the Lua environment:

#### `app` table

- `app.reload()`: Reloads the UI.
- `app.get_widget_by_id(id)`: Returns the widget with the specified `id`.
- `app.windows`: A table of all the windows created by the application, indexed by their titles.
- `app.on_ready`: A function that is called after the UI is built.
- `app.markdown_to_pango(markdown)`: Converts a Markdown string to a Pango markup string.
- `app.on_notification(notification)`: A function that is called when a notification is received. The `notification` argument is a table containing the notification data (`app_name`, `summary`, `body`, `timeout`).
- `app.tray`: A table for managing the system tray. See the `Tray API` section for more details.

#### `Clipboard` table

- `Clipboard.set_text(text)`: Sets the clipboard text.
- `Clipboard.read_text(callback)`: Asynchronously reads the clipboard text and calls the `callback` function with the result.

#### Utility Functions

- `build_ui(config)`: Builds a UI from a Lua table.
- `notify(summary, body)`: Displays a desktop notification.
- `exit(code)`: Exits the application.
- `set_interval(ms, callback)`: Calls the `callback` function every `ms` milliseconds.
- `set_timeout(ms, callback)`: Calls the `callback` function after `ms` milliseconds.
- `exec(cmd)`: Executes a shell command and returns the output.
- `exec_async(cmd, callback)`: Executes a shell command asynchronously and calls the `callback` function with the result.
- `fetch(method, uri, headers, body)`: Performs an HTTP request.
- `fetch_async(method, uri, headers, body, callback)`: Performs an HTTP request asynchronously and calls the `callback` function with the result.
- `spawn(cmd)`: Spawns a new process.
- `graphemes(s)`: Splits a string into a table of its grapheme clusters.

### Services

Ink provides a number of services that can be accessed from your Lua scripts.

#### `Apps` service

- `Apps.list()`: Returns a list of all installed applications. Each application object has the following fields and methods:
  - `name`: The display name of the application.
  - `executable`: The path to the application's executable.
  - `icon`: The name of the application's icon.
  - `launch()`: A method to launch the application.

#### `Audio` service

- `Audio.get_volume()`: An asynchronous function that returns the current volume percentage of the default sink.
- `Audio.set_volume(percent)`: An asynchronous function that sets the volume of the default sink to the specified percentage.
- `Audio.watch(callback)`: A function that watches for volume changes and calls the `callback` function when a change is detected.

#### `fs` service

- `Files.read_file(path)`: Reads the contents of a file.
- `Files.write_file(path, content)`: Writes content to a file.
- `Files.exists(path)`: Checks if a file or directory exists.
- `Files.watch(path, callback)`: Watches a file for changes and calls the `callback` function when the file is modified. Returns a watcher object with a `disconnect()` method.

#### `json` service

- `app.json.parse(json)`: Parses a JSON string and returns a Lua table.
- `app.json.stringify(table)`: Converts a Lua table to a JSON string.

#### `System` service

- `System.get_battery()`: Returns a table with `capacity` and `status` of the battery.
- `System.get_wifi_ssid()`: An asynchronous function that returns the current Wi-Fi SSID.
- `System.set_clipboard(text)`: An asynchronous function that sets the clipboard text.
- `System.media_info()`: An asynchronous function that returns a table with `title` and `artist` of the currently playing media.

#### `tray` service

- `app.tray.get_item_properties(service)`: Returns a table of processed properties for the specified tray item.
- `app.tray.get_item_raw_properties(service)`: Returns a table of raw properties for the specified tray item.
- `app.tray.on_item_added(service)`: A function that is called when a new system tray item is registered. The `service` argument is the name of the new tray item's service.

### LuaAdjustment Object

The `LuaAdjustment` object is returned by `widget:get_vadjustment()`. It has the following methods:

- `get_value()`: Gets the value of the adjustment.
- `set_value(value)`: Sets the value of the adjustment.
- `get_upper()`: Gets the upper value of the adjustment.

## Available Gtk Widget Types

Most of these are untested. You are free to make a PR if you encounter any issues.
I may not be able to update this tab from time to time, so just checkout the file at `src/ui/catalog.rs`.

### Containers

- `GtkWindow`
- `GtkApplicationWindow`
- `GtkDialog`
- `GtkAboutDialog`
- `GtkMessageDialog`
- `GtkColorChooserDialog`
- `GtkFileChooserDialog`
- `GtkFontChooserDialog`
- `GtkAppChooserDialog`
- `GtkBox`
- `GtkCenterBox`
- `GtkListBox`
- `GtkFlowBox`
- `GtkStack`
- `GtkOverlay`
- `GtkPaned`
- `GtkExpander`
- `GtkRevealer`
- `GtkScrolledWindow`
- `GtkViewport`
- `GtkActionBar`
- `GtkHeaderBar`
- `GtkNotebook`
- `GtkFrame`
- `GtkAspectFrame`
- `GtkWindowHandle`
- `GtkPopover`
- `GtkButton`
- `GtkToggleButton`
- `GtkLinkButton`
- `GtkMenuButton`
- `GtkCheckButton`

### Leaf Widgets

- `GtkLabel`
- `GtkImage`
- `GtkPicture`
- `GtkSpinner`
- `GtkProgressBar`
- `GtkLevelBar`
- `GtkCalendar`
- `GtkVideo`
- `GtkSeparator`
- `GtkStatusbar`
- `GtkInfoBar`
- `GtkEntry`
- `GtkPasswordEntry`
- `GtkSearchEntry`
- `GtkSpinButton`
- `GtkSwitch`
- `GtkScale`
- `GtkRange`
- `GtkTextView`
- `GtkColorButton`
- `GtkFontButton`
- `GtkDropDown`
- `GtkComboBox`
- `GtkComboBoxText`
- `GtkListView`
- `GtkGridView`
- `GtkColumnView`
- `GtkDrawingArea`
- `GtkGLArea`
- `GtkEmojiChooser`
- `GtkShortcutsShortcut`
- `GtkLockButton`
- `GtkPopoverMenu`
- `GtkPopoverMenuBar`

## Examples

### Gemini Widget

![Gemini AI Widget](examples/gemini/gemini.png)

### Weather Widget

![Weather Widget](examples/weather/weather.png)

There are several more in the `examples/` directory

## Additional Notes

Error handling is inconsistent. You might find cases where it misses an error it should have caught.

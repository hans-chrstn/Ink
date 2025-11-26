local function toggle_launcher()
	local launcher = app.windows.Launcher
	if launcher then
		local is_visible = launcher:is_visible()
		launcher:set_visible(not is_visible)
	else
		print("Launcher window not found!")
	end
end

local main_file_path = INK_MAIN_FILE_PATH
if main_file_path then
	print("Watching config file for changes: " .. main_file_path)
	Files.watch(main_file_path, function(event)
		if event == "changed" then
			print("Config file changed! Reloading...")
			app.reload()
		end
	end)
end

return {
	{
		title = "Bar",
		default_height = 40,
		window_mode = "layer_shell",
		layer = "top",
		anchors = { top = true, left = true, right = true, bottom = false },
		properties = {
			visible = true,
		},
		children = {
			{
				type = "GtkBox",
				properties = {
					orientation = "horizontal",
					spacing = 10,
					hexpand = true,
					halign = "center",
					valign = "center",
				},
				children = {
					{
						type = "GtkButton",
						properties = {
							label = "Toggle Launcher",
						},
						signals = {
							clicked = toggle_launcher,
						},
					},
				},
			},
		},
	},
	{
		title = "Launcher",
		default_width = 400,
		default_height = 300,
		window_mode = "layer_shell",
		layer = "overlay",
		properties = {
			visible = false,
		},
		children = {
			{
				type = "GtkLabel",
				properties = {
					label = "This is the Launcher window! Edit this text and save to see hot reload.",
					halign = "center",
					valign = "center",
					hexpand = true,
					vexpand = true,
				},
			},
		},
	},
}

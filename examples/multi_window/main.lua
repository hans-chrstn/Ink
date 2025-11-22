return {
	{
		title = "First Window",
		default_width = 300,
		default_height = 200,
		anchors = { top = true, left = true, right = true },
		auto_exclusive_zone = true,
		window_mode = "layer_shell",
		layer = "top",
		children = {
			{
				type = "GtkLabel",
				properties = {
					label = "This is the first window.",
				},
			},
		},
	},
	{
		title = "Second Window",
		default_width = 350,
		default_height = 250,
		anchors = { top = false, left = true, right = true, bottom = true },
		auto_exclusive_zone = true,
		window_mode = "layer_shell",
		layer = "top",
		children = {
			{
				type = "GtkLabel",
				properties = {
					label = "This is the second window.",
				},
			},
		},
	},
}

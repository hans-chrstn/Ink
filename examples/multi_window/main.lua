return {
	{
		anchors = { top = true, left = true, right = true },
		auto_exclusive_zone = true,
		window_mode = "layer_shell",
		layer = "top",
		properties = {
			title = "First Window",
			visible = true,
			default_width = 300,
			default_height = 200,
		},
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
		anchors = { top = false, left = true, right = true, bottom = true },
		auto_exclusive_zone = true,
		window_mode = "layer_shell",
		layer = "top",
		properties = {
			title = "Second Window",
			visible = true,
			default_width = 350,
			default_height = 250,
		},
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

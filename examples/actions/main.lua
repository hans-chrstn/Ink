return {
	type = "GtkApplicationWindow",
	properties = {
		default_width = 400,
		default_height = 300,
		visible = true,
	},

	actions = {
		{
			name = "quit",
			callback = function()
				exit()
			end,
		},
		{
			name = "about",
			callback = function()
				notify("About Ink", "This is an example of GtkActions.")
			end,
		},
	},

	menu = {
		{
			label = "File",
			submenu = {
				{ label = "About", action = "app.about" },
				{ label = "Quit", action = "app.quit" },
			},
		},
		{
			label = "Help",
			submenu = {
				{ label = "Online Docs", action = "app.about" },
			},
		},
	},
	children = {
		{
			type = "GtkBox",
			properties = {
				orientation = "vertical",
				spacing = 20,
				halign = "center",
				valign = "center",
			},
			children = {
				{
					type = "GtkLabel",
					properties = {
						label = "Check the application menu in the top bar!",
					},
				},
				{
					type = "GtkButton",
					properties = {

						action_name = "app.quit",
						label = "Quit via Action",
					},
				},
			},
		},
	},
}

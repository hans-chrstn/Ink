return {
	title = "System Notification Test",
	default_width = 400,
	default_height = 200,
	children = {
		{
			type = "GtkBox",
			properties = {
				orientation = "vertical",
				valign = "center",
				halign = "center",
				spacing = 12,
			},
			children = {
				{
					type = "GtkLabel",
					properties = {
						label = "Click the button to send a system notification.",
					},
				},
				{
					type = "GtkButton",
					properties = {
						label = "Send System Notification",
					},
					signals = {
						clicked = function()
							notify("System Notification", "This is a system-level notification.")
						end,
					},
				},
			},
		},
	},
}

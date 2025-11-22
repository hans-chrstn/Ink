local notifications = require("notifications")

return {
	title = "Custom Notifications Test",
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
						label = "Click the button to show a custom notification.",
					},
				},
				{
					type = "GtkButton",
					properties = {
						label = "Show Custom Notification",
					},
					signals = {
						clicked = function()
							notifications.show("Ink Notification", "This is a custom widget notification.")
						end,
					},
				},
			},
		},
	},
}

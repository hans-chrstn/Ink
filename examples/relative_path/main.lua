local lib = require("lib")

return {
	title = "Relative Path Test",
	default_width = 400,
	default_height = 200,
	children = {
		{
			type = "GtkLabel",
			properties = {
				label = lib.message,
			},
		},
	},
}

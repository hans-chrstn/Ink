local lib = require("lib")
return {
	properties = {
		visible = true,
		default_width = 400,
		default_height = 200,
	},
	children = {
		{
			type = "GtkLabel",
			properties = {
				label = lib.message,
			},
		},
	},
}

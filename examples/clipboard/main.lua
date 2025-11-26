return {
	type = "GtkApplicationWindow",
	properties = {
		title = "Native Clipboard Example",
		default_width = 400,
		default_height = 300,
		visible = true,
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
						label = "Enter text below and use the buttons to interact with the clipboard.",
					},
				},
				{

					type = "GtkEntry",
					properties = {
						name = "clipboard-entry",
						placeholder_text = "Text goes here...",
					},
				},
				{
					type = "GtkBox",
					properties = {
						orientation = "horizontal",
						spacing = 12,
						halign = "center",
					},
					children = {
						{
							type = "GtkButton",
							properties = {
								label = "Copy to Clipboard",
							},
							signals = {
								clicked = function(self)
									local window = self:get_ancestor(Gtk.Window)
									local entry = window:find_child("clipboard-entry")
									if entry then
										Clipboard.set_text(entry:get_text())
										notify("Clipboard", "Text copied!")
									end
								end,
							},
						},
						{
							type = "GtkButton",
							properties = {
								label = "Paste from Clipboard",
							},
							signals = {
								clicked = function(self)
									local window = self:get_ancestor(Gtk.Window)
									local entry = window:find_child("clipboard-entry")
									if entry then
										Clipboard.read_text(function(text)
											if text then
												entry:insert_text(text)
												notify("Clipboard", "Text pasted!")
											else
												notify(
													"Clipboard",
													"Clipboard was empty or contained non-text content."
												)
											end
										end)
									end
								end,
							},
						},
					},
				},
			},
		},
	},
}

local tray_items = {}
local tray_container = nil

function app.tray.on_item_added(service)
	set_timeout(100, function()
		local props = app.tray.get_item_properties(service)
		if not props then
			return
		end

		local title = props.title
		if title == nil or title == "" then
			title = service
		end

		local image_props = {
			pixel_size = 24,
		}

		if props.icon_path then
			image_props.file = props.icon_path
		elseif props.icon_name then
			image_props.icon_name = props.icon_name
		end

		local item_widget = build_ui({
			type = "GtkBox",
			properties = {
				spacing = 5,
				margin_start = 5,
				margin_end = 5,
			},
			children = {
				{
					type = "GtkImage",
					properties = image_props,
				},
				{
					type = "GtkLabel",
					properties = {
						label = title,
					},
				},
			},
		})

		if tray_container then
			tray_container:add(item_widget)
			tray_items[service] = item_widget
		end
	end)
end

function app.tray.on_item_removed(service)
	if tray_items[service] then
		tray_items[service]:destroy()
		tray_items[service] = nil
	end
end

return {
	{
		window_mode = "layer_shell",
		layer = "top",
		anchors = { top = true, right = true },
		properties = {
			title = "Tray Bar",
			visible = true,
			default_height = 40,
		},
		children = {
			{
				type = "GtkBox",
				properties = {
					orientation = "horizontal",
					halign = "end",
					valign = "center",
				},
				signals = {
					map = function(self)
						tray_container = self
					end,
				},
			},
		},
	},
}

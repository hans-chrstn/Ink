local active_notifications = {}

function app.on_notification(params)
	local notif_ui = {
		type = "GtkWindow",
		window_mode = "layer_shell",
		layer = "top",
		anchors = { top = true, right = true, bottom = false, left = false },
		margins = { top = 20, right = 20 },
		properties = {
			visible = true,
		},
		children = {
			{
				type = "GtkBox",
				properties = {
					orientation = "vertical",
					spacing = 10,
					margin_top = 10,
					margin_bottom = 10,
					margin_start = 15,
					margin_end = 15,
				},
				children = {
					{
						type = "GtkLabel",
						properties = {
							label = "<b>" .. params.summary .. "</b>",
							use_markup = true,
							halign = "start",
							css_classes = { "summary" },
						},
					},
					{
						type = "GtkLabel",
						properties = {
							label = params.body,
							halign = "start",
							wrap = true,
							css_classes = { "body" },
						},
					},
				},
			},
		},
	}

	local notif_window = build_ui(notif_ui)

	local id = #active_notifications + 1
	active_notifications[id] = notif_window

	local timeout_ms = (params.timeout > 0 and params.timeout) or 5000
	set_timeout(timeout_ms, function()
		if active_notifications[id] then
			active_notifications[id]:destroy()
			active_notifications[id] = nil
		end
	end)
end

return {
	css = [[
        window {
            background-color: rgba(30, 30, 40, 0.9);
            border-radius: 12px;
            border: 1px solid rgba(120, 120, 150, 0.8);
        }
        label.summary {
            font-size: 1.1em;
            color: #eeeeee;
        }
        label.body {
            font-size: 1.0em;
            color: #cccccc;
        }
    ]],
}

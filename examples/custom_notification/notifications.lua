local notifications = {}

local active_notifications = {}
local start_y = 10

function notifications.show(summary, body)
	local notif_height = 50
	local y = start_y

	for _, notif in ipairs(active_notifications) do
		y = y + notif_height + 10
	end

	local notification_widget

	notification_widget = build_ui({
		window_mode = "layer_shell",
		layer = "top",
		anchors = {
			top = true,
			right = true,
		},
		margins = {
			top = y,
			right = 10,
		},
		children = {
			{
				type = "GtkBox",
				properties = {
					orientation = "vertical",
					spacing = 6,
				},
				children = {
					{
						type = "GtkLabel",
						properties = {
							label = "<b>" .. summary .. "</b>",
							use_markup = true,
							halign = "start",
						},
					},
					{
						type = "GtkLabel",
						properties = {
							label = body,
							halign = "start",
						},
					},
					{
						type = "GtkButton",
						properties = {
							label = "Dismiss",
						},
						signals = {
							clicked = function()
								notification_widget:destroy()
							end,
						},
					},
				},
			},
		},
	})

	table.insert(active_notifications, notification_widget)

	set_timeout(5000, function()
		if notification_widget then
			notification_widget:destroy()
		end
		for i, notif in ipairs(active_notifications) do
			if notif == notification_widget then
				table.remove(active_notifications, i)
				break
			end
		end
	end)
end

return notifications

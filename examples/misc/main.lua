local config = {
	type = "GtkWindow",
	id = "my_test_window_id", -- useful for app.get_widget_by_id('your-id-here')
	properties = {
		title = "Test Window",
		default_width = 400,
		default_height = 300,
		visible = false, -- Start as invisible to test setting and getting visibility
		name = "my_test_window", -- unique identifier for targetting in CSS via # selector (e.g., label#name) or for parent:find_child('name')
		margin_top = 10,
		margin_bottom = 10,
		margin_start = 10,
		margin_end = 10,
	},
	children = {
		{
			type = "GtkLabel",
			id = "my_test_label_id",
			properties = {
				label = "Hello, get_property!",
				name = "test_label",
			},
		},
	},
}

local function verify_properties()
	local accessed_window = app.windows["Test Window"]
	if accessed_window then
		print("Window 'Test Window' successfully found in app.windows.")
	else
		print("Window 'Test Window' NOT found in app.windows. Falling back to ID lookup.")
		accessed_window = app.get_widget_by_id("my_test_window_id")
	end

	if not accessed_window then
		print("Error: Window 'Test Window' not found.")
		return
	end

	print("Window Title: " .. accessed_window:get_property("title"))
	print("Window Default Width: " .. accessed_window:get_property("default-width"))
	print("Window Default Height: " .. accessed_window:get_property("default-height"))
	print("Window Is Visible: " .. tostring(accessed_window:get_property("visible")))
	print("Window name (widget_name property): " .. accessed_window:get_property("name")) -- GTK property 'name' is widget_name

	local non_existent = accessed_window:get_property("non-existent-property")
	if non_existent == nil then
		print("Non-existent property returns nil (correct)")
	else
		print("Non-existent property returns: " .. tostring(non_existent) .. " (incorrect)")
	end

	local test_label = accessed_window:find_child("test_label")
	if not test_label then
		print("Error: Label 'test_label' not found via find_child.")
		test_label = app.get_widget_by_id("my_test_label_id")
	end

	if test_label then
		print("Label text: " .. test_label:get_property("label"))
		print("Label name (widget_name property): " .. test_label:get_property("name"))
	else
		print("Error: Label 'test_label' not found.")
	end

	accessed_window:set_property("visible", true)
	print("Window Is Visible after set_property: " .. tostring(accessed_window:get_property("visible")))

	set_timeout(3000, function()
		accessed_window:destroy()
		print("Window destroyed.")
	end)
end

set_timeout(100, verify_properties)

return config

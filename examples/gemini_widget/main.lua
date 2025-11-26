local thinking_animation
local animation_state = 1
local stop_thinking_animation = false

local function get_api_key()
	local api_key = nil
	if api_key == nil then
		print("Error: API Key not set, change the api_key at :6")
	end
	return api_key
end

local function scroll_to_bottom()
	set_timeout(100, function()
		local scrolled_window = ink.get_widget_by_id("scrolled_window")
		if not scrolled_window then
			return
		end
		local vadjustment = scrolled_window:get_vadjustment()
		if vadjustment then
			vadjustment:set_value(vadjustment:get_upper())
		end
	end)
end

local function add_message(text, sender, id)
	local message_box = ink.get_widget_by_id("message_box")
	if not message_box then
		return nil
	end

	local message = build_ui({
		type = "GtkLabel",
		id = id,
		properties = {
			label = text,
			wrap = true,
			css_classes = { "message", sender .. "-message" },
			halign = sender == "user" and "end" or "start",
		},
	})
	if not message then
		return nil
	end

	message:set_visible(true)
	message_box:add(message, {})
	scroll_to_bottom()
	return message
end

local function update_thinking_indicator()
	if stop_thinking_animation then
		return false
	end

	local indicator = ink.get_widget_by_id("thinking_indicator")
	if not indicator then
		return false
	end

	if animation_state == 1 then
		indicator:set_text("Thinking.")
		animation_state = 2
	elseif animation_state == 2 then
		indicator:set_text("Thinking..")
		animation_state = 3
	else
		indicator:set_text("Thinking...")
		animation_state = 1
	end
	return true
end

local function send_message()
	local entry_input = ink.get_widget_by_id("entry_input")
	local send_button = ink.get_widget_by_id("send_button")
	if not entry_input or not send_button then
		return
	end

	local message_text = entry_input:get_text()
	if message_text == "" then
		return
	end

	entry_input:set_property("sensitive", false)
	send_button:set_property("sensitive", false)

	add_message(message_text, "user")
	entry_input:set_text("")

	stop_thinking_animation = false
	local indicator_widget = add_message("Thinking...", "gemini", "thinking_indicator")
	set_interval(500, update_thinking_indicator)
	local api_key = get_api_key()
	if not api_key then
		if indicator_widget then
			indicator_widget:set_text("Error: API key not configured.")
		end
		stop_thinking_animation = true
		entry_input:set_property("sensitive", true)
		send_button:set_property("sensitive", true)
		return
	end

	local url = "https://generativelanguage.googleapis.com/v1beta/models/gemini-2.0-flash:generateContent"
	local headers = {
		["Content-Type"] = "application/json",
		["X-goog-api-key"] = api_key,
	}
	local body = ink.json.stringify({
		contents = {
			{
				parts = {
					{ text = message_text },
				},
			},
		},
	})

	fetch_async("POST", url, headers, body, function(response)
		stop_thinking_animation = true
		indicator_widget:destroy()

		local response_text_to_add = nil

		if response.err then
			response_text_to_add = "Error: " .. response.err
		else
			local data = ink.json.parse(response.ok)

			if data and data.candidates and #data.candidates > 0 then
				response_text_to_add = data.candidates[1].content.parts[1].text
			else
				response_text_to_add = "Error: Invalid response from Gemini API."
			end
		end

		add_message(response_text_to_add, "gemini")
		scroll_to_bottom()

		entry_input:set_property("sensitive", true)
		send_button:set_property("sensitive", true)
	end)
end

return {
	type = "GtkApplicationWindow",
	id = "main_window",
	properties = {
		title = "Gemini AI",
		default_width = 400,
		default_height = 600,
		visible = true,
	},
	css = [[
        window {
            background-color: #1e1e2e;
        }
        .top-bar {
            background-color: #181825;
            padding: 5px;
            border-bottom: 1px solid #313244;
        }
        .profile-pic {
            border-radius: 50%;
        }
        .bottom-bar {
            background-color: #181825;
            padding: 5px;
            border-top: 1px solid #313244;
        }
        .beautiful-button {
            background-color: #89b4fa;
            color: #1e1e2e;
            border-radius: 5px;
            padding: 10px 20px;
            font-size: 16px;
            font-weight: bold;
        }
        .entry {
            min-height: 40px;
            padding: 5px;
            background-color: #313244;
            color: #cdd6f4;
            border: 1px solid #45475a;
            border-radius: 5px;
        }
        scrolledwindow {
            background-color: #1e1e2e;
            padding: 5px;
        }
        .message {
            margin: 5px;
            padding: 10px;
            border-radius: 10px;
            color: #cdd6f4;
        }
        .user-message {
            background-color: #89b4fa;
            color: #1e1e2e;
        }
        .gemini-message {
            background-color: #45475a;
        }
        label {
            color: #cdd6f4;
        }
        entry {
            color: #cdd6f4;
        }
        .exit-button {
            background-color: #f38ba8;
            border-radius: 12px;
        }
    ]],
	children = {
		{
			type = "GtkBox",
			properties = {
				orientation = "vertical",
				spacing = 0,
			},
			children = {
				{
					type = "GtkBox",
					css_classes = { "top-bar" },
					properties = {
						orientation = "horizontal",
						spacing = 10,
						margin_top = 5,
						margin_bottom = 5,
						margin_start = 10,
						margin_end = 10,
					},
					children = {
						{
							type = "GtkDrawingArea",
							css_classes = { "profile-pic" },
							properties = {
								width_request = 32,
								height_request = 32,
							},
							draw = function(self, cr, width, height)
								cr.set_source_rgb(0.4, 0.6, 0.9)
								cr.arc(width / 2, height / 2, width / 2, 0, 2 * math.pi)
								cr.fill()
							end,
						},
						{
							type = "GtkLabel",
							properties = {
								label = "<b>Gemini AI</b>",
								use_markup = true,
								halign = "start",
								hexpand = true,
							},
						},
						{
							type = "GtkButton",
							properties = {
								width_request = 24,
								height_request = 24,
							},
							css_classes = { "exit-button" },
							signals = {
								clicked = function()
									exit()
								end,
							},
						},
					},
				},
				{
					type = "GtkScrolledWindow",
					id = "scrolled_window",
					properties = {
						hscrollbar_policy = "never",
						vscrollbar_policy = "automatic",
						vexpand = true,
					},
					children = {
						{
							type = "GtkBox",
							id = "message_box",
							properties = {
								orientation = "vertical",
								spacing = 5,
								margin_top = 10,
								margin_bottom = 10,
								margin_start = 10,
								margin_end = 10,
							},
						},
					},
				},
				{
					type = "GtkBox",
					css_classes = { "bottom-bar" },
					properties = {
						orientation = "horizontal",
						spacing = 10,
						margin_top = 5,
						margin_bottom = 5,
						margin_start = 10,
						margin_end = 10,
					},
					children = {
						{
							type = "GtkEntry",
							id = "entry_input",
							properties = {
								placeholder_text = "Enter a prompt...",
								hexpand = true,
							},
							css_classes = { "entry" },
							signals = {
								activate = send_message,
							},
						},
						{
							type = "GtkButton",
							id = "send_button",
							properties = {
								label = "Send",
								valign = "center",
							},
							css_classes = { "beautiful-button" },
							signals = {
								clicked = send_message,
							},
						},
					},
				},
			},
		},
	},
}

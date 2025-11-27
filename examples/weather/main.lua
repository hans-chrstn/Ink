local app = app

local icn = require("icons")

local weather_widget_id = "weather_info_label"
local city_entry_id = "city_input_entry"
local weather_icon_id = "weather_icon_label"

local last_fetch_time = 0
local MIN_REFRESH_INTERVAL_SECONDS = 300 -- 5 minutes

local function update_weather(city_param)
	local city_to_fetch = city_param or "London"

	local current_time = os.time()

	last_fetch_time = current_time

	local city_input_entry_widget = app.get_widget_by_id(city_entry_id)
	if city_input_entry_widget then
		city_input_entry_widget:set_text(city_to_fetch)
	end

	local label_widget = app.get_widget_by_id(weather_widget_id)
	local icon_widget = app.get_widget_by_id(weather_icon_id)

	if label_widget then
		label_widget:set_text("Fetching weather for " .. city_to_fetch .. "...")
	end
	if icon_widget then
		icon_widget:set_text(icn["default"])
	end

	fetch_async("GET", "https://wttr.in/" .. city_to_fetch .. "?format=j1", nil, nil, function(result)
		local display_text = "Error fetching weather data."
		local icon_to_display = icn["default"]

		if result.ok then
			local json_data = app.json.parse(result.ok)
			if json_data and json_data.current_condition and json_data.current_condition[1] then
				local current = json_data.current_condition[1]
				local temp_c = current.temp_C
				local desc = current.weatherDesc[1].value
				local humidity = current.humidity
				local wind_speed = current.windspeedKmph
				local weather_code = tostring(current.weatherCode)

				icon_to_display = icn[weather_code] or icn["default"]

				local city_graphemes = graphemes(city_to_fetch)
				local city_display = table.concat(city_graphemes, " ")

				display_text = string.format(
					"%s\n%s°C, %s\nHumidity: %s%%\nWind: %s km/h",
					city_display,
					temp_c,
					desc,
					humidity,
					wind_speed
				)
			else
				display_text = "Could not parse weather data for " .. city_to_fetch
			end
		else
			display_text = "Error fetching weather for " .. city_to_fetch .. ": " .. result.err
		end

		if label_widget then
			label_widget:set_text(display_text)
		end
		if icon_widget then
			icon_widget:set_text(icon_to_display)
		end
	end)
end

app.on_ready = function()
	update_weather("London")

	set_interval(MIN_REFRESH_INTERVAL_SECONDS * 1000, function()
		local city_from_entry = app.get_widget_by_id(city_entry_id):get_text()
		update_weather(city_from_entry)
		return true
	end)
end

return {
	type = "GtkApplicationWindow",
	properties = {
		title = "Weather Widget",
		default_width = 350,
		default_height = 300,
		visible = true,
	},
	css = [[
        window {
            background-color: #2e3440; /* Nord dark blue */
            border-radius: 8px;
        }
        label {
            color: #eceff4; /* Nord light text */
            font-family: 'Comic Sans MS', sans-serif;
        }
        .weather-display {
            font-size: 16px;
            font-weight: bold;
            text-align: center;
        }
        .title {
            text-align: center;
        }
        entry {
            padding: 5px;
            border-radius: 4px;
            background-color: #3b4252; /* Nord darker gray */
            color: #eceff4;
            border: 1px solid #4c566a;
        }
        button {
            background-color: #5e81ac; /* Nord blue */
            color: #eceff4;
            border-radius: 4px;
            padding: 8px 12px;
            font-weight: bold;
        }
        button:hover {
            background-color: #81a1c1; /* Nord lighter blue */
        }
        .weather-icon {
            font-size: 64px; /* Larger icon */
            text-align: center;
        }
    ]],
	children = {
		{
			type = "GtkBox",
			properties = {
				orientation = "vertical",
				spacing = 15,
				margin_top = 15,
				margin_bottom = 15,
				margin_start = 15,
				margin_end = 15,
			},
			children = {
				{
					type = "GtkLabel",
					properties = {
						label = "<b>Weather Forecast</b>",
						use_markup = true,
						xalign = 0.5,
						css_classes = { "title" },
					},
				},
				{
					type = "GtkBox",
					properties = {
						orientation = "horizontal",
						spacing = 10,
						halign = "center",
						valign = "center",
					},
					children = {
						{
							type = "GtkLabel",
							id = weather_icon_id,
							properties = {
								label = icn["default"],
								css_classes = { "weather-icon" },
							},
						},
						{
							type = "GtkLabel",
							id = weather_widget_id,
							properties = {
								label = "Fetching weather...",
								wrap = true,
								justify = "center",
								css_classes = { "weather-display" },
								hexpand = true,
								vexpand = true,
							},
						},
					},
				},
				{
					type = "GtkSeparator",
					properties = {
						orientation = "horizontal",
						margin_top = 5,
						margin_bottom = 5,
					},
				},
				{
					type = "GtkLabel",
					properties = {
						label = "Enter City:",
						xalign = 0,
					},
				},
				{
					type = "GtkEntry",
					id = city_entry_id,
					properties = {
						text = "London",
					},
				},
			},
		},
	},
	signals = {
		close_request = function()
			app.quit()
		end,
	},
}

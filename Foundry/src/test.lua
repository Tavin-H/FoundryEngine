local speed = 0.001
--local id = id:this()

function update()
	if input:get_key("KeyW") then
		transform.Translate(0, Vec3(-1, -1, 0) * speed)
	end
	if input:get_key("KeyS") then
		transform.Translate(0, Vec3(1, 1, 0.0) * speed)
	end
	if input:get_key("KeyA") then
		transform.Translate(0, Vec3(1, -1, 0.0) * speed)
	end
	if input:get_key("KeyD") then
		transform.Translate(0, Vec3(-1, 1, 0.0) * speed)
	end
end

function on_broadcast(message)
	if message == "Hello" then
		print("Hi there")
	end
end

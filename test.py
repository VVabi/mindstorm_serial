import paho.mqtt.client as mqtt


client = mqtt.Client()

client.connect("localhost", 1883, 60)
input("Press Enter to continue...")
client.publish("motor/register", '{"port": "A"}')
client.publish("motor/register", '{"port": "B"}')
#print("Publish")
#client.publish("motor/set_pwm", '{"commands": [{"port": "A", "pwm": 0}, {"port": "B", "pwm": 0}]')
#print("Done Publish")
client.publish("motor/goto_position", '{"port": "A", "position": 90}')
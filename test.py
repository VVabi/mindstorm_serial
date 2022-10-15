import paho.mqtt.client as mqtt


client = mqtt.Client()

client.connect("localhost", 1883, 60)
client.publish("motor/register", '{"port": "A"}')
client.publish("motor/register", '{"port": "A"}')
#print("Publish")
client.publish("motor/set_pwm", '{"commandsx": [{"port": "A", "pwm": 0}]}')
#print("Done Publish")
#client.publish("motor/goto_position", '{"port": "A", "position": 180}')
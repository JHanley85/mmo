#!/bin/bash
watch tail -f $(/bin/ls -1t /opt/ts3server/Examples/server/logs | /bin/sed q)
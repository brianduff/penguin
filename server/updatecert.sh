#!/bin/bash

sudo certbot -d '*.dufflin.com' --manual --preferred-challenges dns certonly
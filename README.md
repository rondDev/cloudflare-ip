# Cloudflare DNS Auto IP Updater
This is a rewrite of [pigeonburger's repo](https://github.com/pigeonburger/cloudflare-ip), I have not tested if it works (due to not having a website to test it with currently). 

Theoretically it should update your IP on Cloudflare, kind of like Dynamic DNS. It checks for updated IP adress every 5 minutes, if your IP is different from the one on Cloudflare's site, it sends an update request to them.

# Usage
### Windows
To use this, navigate to [Releases](https://github.com/rondDev/cloudflare-ip/releases/tag/v1) and download the binary.

### Other
You will need to compile it yourself. Please look up a guide if you can't figure it out.

# Credits
### [pigeonburger](https://github.com/pigeonburger) - For creating the original repository

Organization - Can distribute agents but agent is precompiled with fingerprint unique to that organization 
Clients - Users that install agent 

When a user install an agent 
- Create a session and unique identifier 
- User is listed under Organization devices 
- Organization can connection to that user pc 


1. Agent don't expose to internet, It's proxied from local to server and then to rdp viewer client 
2. Agent is a service that gets restart on window startup.
3. when installation start on mac a ui for permission setup is configured. 
4. Windows admin access is requested for that executable and then service is installed and executable deleted.

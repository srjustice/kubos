Communications Service
======================

Communication services serve as the connection between the radio hardware interface and the rest
of the flight software. Communication services are integrated into radio hardware services so that
they provide a tunnel for data to travel through utilizing a specific radio's API while also 
providing a queryable endpoint for GraphQL queries and mutations to interact with. 

An ethernet service provides an example of such integration and services as a mock radio interface.

Comms Service Control Block
---------------------------

The comms service control block is used to pass a specific configuration details into a 
communications service. The control block can be filled manually or with the help of a 
configuration CommsConfig struct that can be used to parse a TOML file to obtain most of the 
configuration details into the control block. Developers will then need to provide connections to
specific interfaces and the read and write functions across those interfaces.

The control block requires filling the following fields:

**read**
  This is an optional function pointer wrapped in an Arc that will read data from a radio, deframe 
  those data packets, and then compose any fragmented packets into a single UDP packet. This 
  function is necessary for developers wanting uplink communication to the radio.

**write**
  This is a vector of function pointers each wrapped in Arcs that fragment a UDP packet as 
  necessary, frame these fragments and then write these fragments across the radio interface for
  downlink communication. At least one write function is required for the communication service to
  work. If multiple functions are provided, the first will be used to downlink any responses to 
  uplinked traffic. All provided write functions will also be used to spawn endpoints that mission 
  applications can write to downlink information.

**read_conn**
  This is the interface connection to the radio that the read function will read from.

**write_conn**
  This is the interface connection to the radio that the write function will write to.

**handler_port_min**
  In order to coordinate communication between the comms service and different services 
  asynchronously, handler threads are spawned to handle individual GraphQL requests. This field 
  describes the lower end of a range of ports reserved for handler threads.

**handler_port_max**
  This field describes the upper end of a range of ports reserved for handler threads.  

**timeout**
  Timeout for the completion of GraphQL operations within message handlers in milliseconds.

**ground_ip**
  The IP address of the ground gateway. This is used to build UDP checksums.

**satellite_ip**
  The IP address of the computer that is running the comms service. This is used to build UDP 
  checksums.

**downlink_ports**
  Ports that are used to spawn downlink endpoints, one for each of different write function 
  provided. The number of ports provided each should match the number of write functions provided.

**ground_port**
  The port which the ground gateway is bound. Used as the destination in downlink UDP packets.

Comms Configuration
-------------------

Developers can use the CommsConfig library to generate easy to utilize structs from TOML files to 
allow developers to quickly reconfigure some details passed into a comms service control block
without needing to recompile the binary of the particular hardware service. 

A complete configuration file will look like the following:

  [ethernet-service]
  handler-port-min = 13002
  handler-port-max = 13010
  downlink-ports = [13011]
  ground-port = 9001
  timeout = 1500
  ground-ip = "192.168.8.1"
  satellite-ip = "192.168.8.2"

Note that all provided fields are optional and will be filled in by default values if they are
formatted incorrectly or missing.

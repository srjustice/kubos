Kubos Applications Service
==========================

The Kubos applications service is responsible for monitoring and managing all mission applications for a system.

.. todo::
    
    TODO: Something something installation, upgrades, and recovery
    
    TODO: User/App/Service interaction diagram

Whenever a new application is registered with the service, its manifest file and all other files in
the specified directory are copied into the service's application registry.
By default, this registry is stored under `/home/system/kubos/apps`.

Each application will be automatically assigned a UUID to be used for identification purposes internally.
Using UUIDs, rather than the application's name, allows users the freedom to adjust the application name as they see fit,
for instance if the overall purpose of the application changes and they would like to update the name to reflect that in later versions.

.. figure:: ../images/app_registry.png
   :alt: Application Registry

Communicating with the Service
------------------------------

The applications service uses the same UDP+GraphQL communication scheme as the :doc:`other services <../services/graphql>`.

Users will send GraphQL queries and mutations to the service's UDP port.
The port number can be found in the systems configuration file in `/home/system/etc/config.toml`

Querying
--------

A current list of all available versions of all registered applications can be generated by using the ``apps`` query.

For example::

    {
        apps {
            active,
            app {
                uuid,
                name,
                version
            }
    }
    
Using our example registry, the data returned by the service would be::

    {
        "apps": [
            { 
                "active": false,
                "app": {
                    "uuid": "46d01f19-ab45-4c6f-896e-88f90266f12e",
                    "name": "main-mission",
                    "version": "1.0"
                }
            },
            { 
                "active": false,
                "app": {
                    "uuid": "46d01f19-ab45-4c6f-896e-88f90266f12e",
                    "name": "main-mission",
                    "version": "1.1"
                }
            },
            { 
                "active": true,
                "app": {
                    "uuid": "46d01f19-ab45-4c6f-896e-88f90266f12e",
                    "name": "main-mission",
                    "version": "2.0"
                }
            },
            { 
                "active": true,
                "app": {
                    "uuid": "60ff7516-a5c4-4fea-bdea-1b163ee9bd7a",
                    "name": "payload-app",
                    "version": "1.0"
                }
            },
        ]
    }

To list all available versions of a specific application, specify the desired UUID as an input parameter.

For example::

    {
        apps(uuid: "60ff7516-a5c4-4fea-bdea-1b163ee9bd7a") {
            app {
                name,
                version
            }
        }
    }
    
.. _register-app:

Registering
-----------

Once an application has been written and compiled, the application and its accompanying :ref:`manifest.toml file <app-manifest>`
should be transferred to a new directory on the OBC.
This file transfer can be done using the :doc:`file transfer service <../services/file>`.

The application may be split into multiple files (which is useful for large Python apps), however,
the name of the initial file which should be called for execution must exactly match the ``name``
property in the manifest file.

It can then be registered with the applications service using the ``register`` mutation by specifying
the directory containing the application files.

The service will copy the application from the specified path into the apps registry.
Once registered, users may delete the original application.

For example::

    mutation {
        register(path: "/home/kubos/payload-app") {
            success,
            errors,
            entry {
                active,
                app {
                    name,
                    version
                }
            }
        }
    }

The ``success`` response field is a boolean value which reflects whether the registration process
completed successfully.

If ``true``, then the ``entry`` field will contain the registration information about the newly
registered application.

If ``false,`` then the ``entry`` field will be empty, and the ``errors`` field will contain an
error message detailing what went wrong.

De-Registering
--------------

A particular version of an application can be removed using the ``uninstall`` mutation.

The mutation returns two fields:

    - ``success`` - Indicating the overall result of the uninstall operation
    - ``errors`` - Any errors which were encountered during the uninstall process

For example::

    mutation {
        uninstall(uuid: "46d01f19-ab45-4c6f-896e-88f90266f12e", version: "1.1") {
            success,
            errors
        }
    }
    
    
.. _start-app:
    
Starting an Application
-----------------------

To manually start an application, the ``startApp`` mutation can be used.

The mutation takes two arguments: the UUID of the application to start and the run level which the
app should execute with.

The mutation will return three fields:

    - ``success`` - Indicating the overall result of the operation
    - ``errors`` - Any errors which were encountered while starting the application
    - ``pid`` - The PID of the started application. This will be empty if any errors are encountered

For example::

    mutation {
        startApp(uuid: "60ff7516-a5c4-4fea-bdea-1b163ee9bd7a", runLevel: "OnCommand") {
            success,
            errors,
            pid
        }
    }
    
Under the covers, the service receives the mutation and identifies the current active version of the
application specified. It then calls that version's binary, passing along the run level as a command argument.

Passing Additional Arguments
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

To pass additional arguments to the underlying application, the ``args`` input argument can be used.

For example::

    mutation {
        startApp(uuid: "60ff7516-a5c4-4fea-bdea-1b163ee9bd7a", runLevel: "OnCommand", args: "--verbose --release") {
            success
        }
    }
    
Under the covers, the application would be called like so::

    mission-app -r OnCommand --verbose --release
    
Automatically Starting on Boot
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

All applications will be started with the ``OnBoot`` run level automatically when the applications service is
started during system initialization.

This logic may also be triggered by manually starting the applications service with the ``-b`` flag.

Upgrading
---------

Users may register a new version of an application without needing to remove the existing registration.

To do this, they will use the ``register`` mutation with the optional ``uuid`` input parameter.
An application's UUID is given as a return field of the ``register`` mutation and can also be looked up
using the ``apps`` query.

::
    
    mutation {
        register(path: /home/kubos/payload-app, uuid: 60ff7516-a5c4-4fea-bdea-1b163ee9bd7a) {
            active,
            app {
                name,
                version
            }
        }
    }
        
        
.. todo::
    
    Recovery
    //--------
    
    Is not a thing that actually exists yet...
    
    TODO: Automatic and manual rollback

Customizing the Applications Service
------------------------------------

The configuration for the applications service is saved in `/home/system/etc/config.toml`.
This file can be editted to add or modify the following fields:

- ``[app-service.addr]``

    - ``ip`` - The IP address that the service will use
    - ``port`` - The UDP port GraphQL requests should be sent to

- ``[app-service]``

    - ``registry-dir`` - *(Default: /home/system/kubos/apps)* The directory under which all registry entries should be stored

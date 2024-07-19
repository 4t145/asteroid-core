actor:


# handlers:
    
typeid -> handler
## for raw(local) message
directly call handler
## for binary(remote) message
deserialize message -> call handler


# broker
(actor_id + message) 
router find the broker


INSERT INTO notification(id, user_id, recipient, channel, template_id, status) 
VALUES ($1, $2, $3, $4, $5, $6);
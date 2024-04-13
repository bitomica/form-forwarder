# form-forwarder

This is a basic HTTP server written in Rust designed to forward POST requests to an email address, primarily used for managing web forms.

Request CURL example:

```sh
curl -d "key={SERVER_KEY}&param1=value1&param2=value21" -X POST http://localhost:{SERVER_PORT}
```

# .env file configuration

Each field is required and expected fields in the `.env` file used by form-forwarder.

```env
SERVER_PORT=server_port_for_form_forwarder
SERVER_KEY=form_forwarder_key
SMTP_SERVER=smtp_server_address
SMTP_PORT=smtp_server_tls_port
SMTP_USER=smtp_server_user
SMTP_PASSWORD=smtp_server_password
SMTP_DOMAIN=domain_used_for_smtp_connection
EMAIL_SENDER=email_sender_from
EMAIL_RCPT=email_recipient
EMAIL_SUBJECT=email_subject
```

## Fields

### `SERVER_PORT`

Server port to be open for incoming HTTP POST requests.

### `SERVER_KEY`

Key used to authorized HTTP POST requests.

### `SMTP_SERVER`

SMTP server addreess used for email sending.

### `SMTP_PORT`

SMTP server port used for email sending.

### `SMTP_USER`

Username used for SMTP server authentication.

### `SMTP_PASSWORD`

Password used for SMTP server authentication.

### `SMTP_DOMAIN`

Domain used for SMTP server connection.

### `EMAIL_SENDER`

Sender email address used for the forwarded email.

### `EMAIL_RCPT`

Recipient email address used for the forwarded email.

### `EMAIL_SUBJECT`

Email subject used for the forwarded email.

CREATE TABLE Notification (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    recipient TEXT NOT NULL,
    channel TEXT CHECK(channel IN('email', 'push', 'sms')),
    template_id UUID,
    status TEXT CHECK(status IN('pending', 'sent', 'failed')),
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE Template (
    id UUID PRIMARY KEY,
    user_id UUID NOT NULL,
    name TEXT NOT NULL,
    type TEXT CHECK(type IN('email', 'push', 'sms')),
    content TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE Users (
    id UUID PRIMARY KEY,
    email TEXT NOT NULL,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE Webhook (
    id UUID PRIMARY KEY,
    user_id UUID,
    url TEXT NOT NULL,
    events JSONB,
    created_at TIMESTAMP DEFAULT NOW()
);

ALTER TABLE Notification ADD CONSTRAINT nt_usr FOREIGN KEY (user_id) REFERENCES Users(id);
ALTER TABLE Template ADD CONSTRAINT tp_usr FOREIGN KEY (user_id) REFERENCES Users(id);
ALTER TABLE Webhook ADD CONSTRAINT wh_usr FOREIGN KEY (user_id) REFERENCES Users(id);
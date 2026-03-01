import React, { FC } from 'react';

interface User {
    id: number;
    name: string;
    email: string;
}

interface UserProfileProps {
    user: User | null;
}

export const UserProfile: FC<UserProfileProps> = ({ user }) => {
    if (!user) {
        return <div>No user found</div>;
    }
    
    return (
        <div className="user-profile">
            <h1>{user.name}</h1>
            <p>{user.email}</p>
            <UserActions userId={user.id} />
        </div>
    );
};

interface UserActionsProps {
    userId: number;
}

const UserActions: FC<UserActionsProps> = ({ userId }) => {
    function handleEdit() {
        console.log(`Editing user ${userId}`);
    }
    
    function handleDelete() {
        console.log(`Deleting user ${userId}`);
    }
    
    return (
        <div className="actions">
            <button onClick={handleEdit}>Edit</button>
            <button onClick={handleDelete}>Delete</button>
        </div>
    );
};

export function UserList() {
    const [users, setUsers] = React.useState<User[]>([]);
    
    return (
        <ul>
            {users.map(user => (
                <li key={user.id}>{user.name}</li>
            ))}
        </ul>
    );
}

# LOMO sample board

- [ ] (p2) #a1b2c3: Implement user authentication middleware
  - Context/reference: New requests need a checked session before they reach protected routes.
  - Locations: `src/auth/`
  - Done when: Protected routes reject a missing or invalid session.
- [.] (p0) #d4e5f6: Fix critical buffer overflow in TCP socket listener
  - Context/reference: A crafted length field can write past the receive buffer.
  - Constraints: Do not change the wire format.
  - Done when: An oversized length is rejected and the listener stays up.
- [+] (p2) #0abc12: Add index to user_sessions table
  - Context/reference: Discovered while tracing session lookup during authentication.
  - Done when: Session lookup uses the index.
- [!] (p0) #789def: Run database schema deletion script
  - Context/reference: The script drops a table that still has production data.
  - Done when: A human has confirmed the deletion and the script has been run.

## Nested

- [.] (p1) #112233: Parent task
  - Context/reference: The child is a separate piece of work, not supporting detail.
  - Done when: The child task is finished.
  - [ ] (p2) #445566: Child stays pending
    - Context/reference: This remains pending while the parent is in progress.
    - Done when: The child has its own completion condition met.

```
- [ ] not a task inside a fence
```

Trailing prose is preserved.

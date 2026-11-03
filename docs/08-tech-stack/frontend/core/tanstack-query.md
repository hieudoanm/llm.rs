# llm — TanStack Query

- `useQuery(['algorithms'])` staleTime 5 minutes.
- `useQuery(['job', id], { refetchInterval: (q) => terminal(q.state.data) ? false : 1000 })`.
- `useMutation` for upload and create job; on success invalidate `['jobs']`.
- Do not poll the history list at 1 s; only the open job.

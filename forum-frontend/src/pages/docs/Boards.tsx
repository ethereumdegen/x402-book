import DocsWrapper from './DocsWrapper'

export default function Boards() {
  return (
    <DocsWrapper title="Boards">
      <p className="lead">
        Boards are topic categories that organize content. Each board has a unique slug and contains threads.
      </p>

      <h2 id="list-boards">List Boards</h2>
      <pre><code>{`GET /boards`}</code></pre>
      <p>Returns all boards with thread counts.</p>

      <h3 id="list-boards-response">Response</h3>
      <pre><code>{`[
  {
    "id": 1,
    "slug": "technology",
    "name": "Technology",
    "description": "AI, software, and the future of tech",
    "nsfw": false,
    "thread_count": 24
  },
  {
    "id": 2,
    "slug": "research",
    "name": "Research",
    "description": "Academic papers and scientific discourse",
    "nsfw": false,
    "thread_count": 12
  }
]`}</code></pre>

      <hr />

      <h2 id="get-board">Get Board</h2>
      <pre><code>{`GET /boards/:slug`}</code></pre>
      <p>Returns a single board by its slug.</p>

      <h3 id="get-board-parameters">Parameters</h3>
      <table>
        <thead>
          <tr>
            <th>Name</th>
            <th>Type</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><code>slug</code></td>
            <td>string</td>
            <td>Board URL slug (e.g., "technology")</td>
          </tr>
        </tbody>
      </table>

      <h3 id="get-board-response">Response</h3>
      <pre><code>{`{
  "id": 1,
  "slug": "technology",
  "name": "Technology",
  "description": "AI, software, and the future of tech",
  "max_threads": 100,
  "nsfw": false
}`}</code></pre>

      <hr />

      <h2 id="board-object">Board Object</h2>
      <table>
        <thead>
          <tr>
            <th>Field</th>
            <th>Type</th>
            <th>Description</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td><code>id</code></td>
            <td>integer</td>
            <td>Unique identifier</td>
          </tr>
          <tr>
            <td><code>slug</code></td>
            <td>string</td>
            <td>URL-friendly identifier</td>
          </tr>
          <tr>
            <td><code>name</code></td>
            <td>string</td>
            <td>Display name</td>
          </tr>
          <tr>
            <td><code>description</code></td>
            <td>string?</td>
            <td>Board description</td>
          </tr>
          <tr>
            <td><code>max_threads</code></td>
            <td>integer?</td>
            <td>Maximum threads allowed</td>
          </tr>
          <tr>
            <td><code>nsfw</code></td>
            <td>boolean</td>
            <td>Whether board allows NSFW content</td>
          </tr>
          <tr>
            <td><code>thread_count</code></td>
            <td>integer</td>
            <td>Number of threads (in list response)</td>
          </tr>
        </tbody>
      </table>
    </DocsWrapper>
  )
}

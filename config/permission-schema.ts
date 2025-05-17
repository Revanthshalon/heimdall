// Define a User namespace
class User implements Namespace {
  related: {
    manager: User[];
    teams: Team[];
  }
}

// Define a Team namespace
class Team implements Namespace {
  related: {
    members: (User | Team)[];
    parent: Team[];
    active: boolean;
  }
}

// Define a Document namespace with permissions
class Document implements Namespace {
  related: {
    owner: User[];
    editors: User[];
    viewers: (User | SubjectSet<Team, "members">)[];
    parent_folder: Folder[];
    confidential: boolean;
  }

  permits: {
    edit: (ctx: Context) => 
      this.related.owner.includes(ctx.subject) || 
      this.related.editors.includes(ctx.subject) ||
      this.related.parent_folder.traverse(parent => 
        parent.permits.edit(ctx)
      );

    view: (ctx) => 
      this.permits.edit(ctx) || 
      this.related.viewers.includes(ctx.subject) ||
      this.related.parent_folder.traverse((parent) => 
        parent.permits.view(ctx)
      );

    share: (ctx) => 
      this.related.owner.includes(ctx.subject) && 
      !this.related.confidential;
  }
}

// Define a Folder namespace
class Folder implements Namespace {
  related: {
    owner: User[];
    editors: SubjectSet<Team, "members">[];
    admins: User[];
  }

  permits: {
    edit: (ctx) => 
      this.related.owner.includes(ctx.subject) || 
      this.related.editors.includes(ctx.subject) ||
      this.related.admins.includes(ctx.subject);

    view: (ctx) => this.permits.edit(ctx);
  }
}


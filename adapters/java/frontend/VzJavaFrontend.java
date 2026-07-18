import java.io.*;
import java.nio.charset.StandardCharsets;
import java.util.*;
import javax.tools.*;
import com.sun.source.tree.*;
import com.sun.source.util.*;

// This is a Java Compiler API client, not a parser. It emits a deliberately
// small, versioned-neutral DTO that the Rust adapter lowers to Semantic IR.
public final class VzJavaFrontend {
  static String q(String s) { return "\"" + s.replace("\\", "\\\\").replace("\"", "\\\"").replace("\n", "\\n").replace("\r", "\\r") + "\""; }
  static String a(List<String> xs) { return "[" + String.join(",", xs) + "]"; }
  static String n(String k, String v) { return "{\"kind\":" + q(k) + (v.isEmpty() ? "" : "," + v) + "}"; }
  static String field(String k, String v) { return q(k) + ":" + v; }

  static String expr(ExpressionTree e) {
    if (e == null) return n("unit", "");
    if (e instanceof IdentifierTree) return n("identifier", field("name", q(((IdentifierTree)e).getName().toString())));
    if (e instanceof LiteralTree) {
      Object v = ((LiteralTree)e).getValue();
      return n("literal", field("value", q(v == null ? "null" : String.valueOf(v))) + "," + field("literal_kind", q(v == null ? "unit" : (v instanceof Boolean ? "boolean" : (v instanceof Number ? "number" : "text")))));
    }
    if (e instanceof BinaryTree) { BinaryTree x=(BinaryTree)e; return n("binary", field("op",q(op(x.getKind())))+","+field("left",expr(x.getLeftOperand()))+","+field("right",expr(x.getRightOperand()))); }
    if (e instanceof UnaryTree) { UnaryTree x=(UnaryTree)e; return n("unary", field("op",q(op(x.getKind())))+","+field("operand",expr(x.getExpression()))); }
    if (e instanceof AssignmentTree) { AssignmentTree x=(AssignmentTree)e; return n("assignment", field("target",expr(x.getVariable()))+","+field("value",expr(x.getExpression()))); }
    if (e instanceof CompoundAssignmentTree) { CompoundAssignmentTree x=(CompoundAssignmentTree)e; return n("mutation", field("op",q(op(x.getKind())))+","+field("target",expr(x.getVariable()))+","+field("value",expr(x.getExpression()))); }
    if (e instanceof MethodInvocationTree) { MethodInvocationTree x=(MethodInvocationTree)e; List<String> xs=new ArrayList<>(); for(ExpressionTree p:x.getArguments()) xs.add(expr(p)); return n("call",field("callee",expr(x.getMethodSelect()))+","+field("arguments",a(xs))); }
    if (e instanceof MemberSelectTree) return n("identifier",field("name",q(((MemberSelectTree)e).getIdentifier().toString())));
    if (e instanceof ParenthesizedTree) return expr(((ParenthesizedTree)e).getExpression());
    if (e instanceof NewArrayTree) { List<String> xs=new ArrayList<>(); if(((NewArrayTree)e).getInitializers()!=null) for(ExpressionTree p:((NewArrayTree)e).getInitializers()) xs.add(expr(p)); return n("collection",field("elements",a(xs))); }
    return n("unsupported", "");
  }
  static String op(Tree.Kind k) {
    switch(k) {
      case PLUS: case PLUS_ASSIGNMENT: return "add"; case MINUS: case MINUS_ASSIGNMENT: return "subtract";
      case MULTIPLY: case MULTIPLY_ASSIGNMENT: return "multiply"; case DIVIDE: case DIVIDE_ASSIGNMENT: return "divide";
      case REMAINDER: return "remainder"; case EQUAL_TO: return "equal"; case NOT_EQUAL_TO: return "not_equal";
      case LESS_THAN: return "less_than"; case GREATER_THAN: return "greater_than";
      case LESS_THAN_EQUAL: return "less_equal"; case GREATER_THAN_EQUAL: return "greater_equal";
      case CONDITIONAL_AND: return "and"; case CONDITIONAL_OR: return "or"; case LOGICAL_COMPLEMENT: return "not"; case UNARY_MINUS: return "negate";
      default: return "unknown";
    }
  }
  static List<String> block(StatementTree s) { if(s instanceof BlockTree) { List<String> xs=new ArrayList<>(); for(StatementTree x:((BlockTree)s).getStatements()) xs.add(stmt(x)); return xs; } return Arrays.asList(stmt(s)); }
  static String stmt(StatementTree s) {
    if (s instanceof VariableTree) { VariableTree x=(VariableTree)s; return n("variable",field("name",q(x.getName().toString()))+","+field("type",q(x.getType()==null?"unknown":x.getType().toString()))+","+field("initializer",expr(x.getInitializer()))); }
    if (s instanceof ExpressionStatementTree) { String x=expr(((ExpressionStatementTree)s).getExpression()); return x; }
    if (s instanceof ReturnTree) return n("return",field("value",expr(((ReturnTree)s).getExpression())));
    if (s instanceof IfTree) { IfTree x=(IfTree)s; return n("if",field("condition",expr(x.getCondition()))+","+field("then",a(block(x.getThenStatement())))+","+field("else",a(x.getElseStatement()==null?Collections.emptyList():block(x.getElseStatement())))); }
    if (s instanceof WhileLoopTree) { WhileLoopTree x=(WhileLoopTree)s; return n("while",field("condition",expr(x.getCondition()))+","+field("body",a(block(x.getStatement())))); }
    if (s instanceof ForLoopTree) { ForLoopTree x=(ForLoopTree)s; List<String> init=new ArrayList<>(); for(StatementTree i:x.getInitializer()) init.add(stmt(i)); List<String> update=new ArrayList<>(); for(ExpressionStatementTree i:x.getUpdate()) update.add(stmt(i)); return n("for",field("init",a(init))+","+field("condition",expr(x.getCondition()))+","+field("update",a(update))+","+field("body",a(block(x.getStatement())))); }
    if (s instanceof EnhancedForLoopTree) { EnhancedForLoopTree x=(EnhancedForLoopTree)s; return n("foreach",field("variable",q(x.getVariable().getName().toString()))+","+field("iterable",expr(x.getExpression()))+","+field("body",a(block(x.getStatement())))); }
    if (s instanceof BlockTree) return n("block",field("body",a(block(s))));
    return n("unsupported", "");
  }
  static String method(MethodTree m) { List<String> ps=new ArrayList<>(); for(VariableTree p:m.getParameters()) ps.add("{"+field("name",q(p.getName().toString()))+","+field("type",q(p.getType().toString()))+"}"); return "{"+field("name",q(m.getName().toString()))+","+field("return_type",q(m.getReturnType()==null?"void":m.getReturnType().toString()))+","+field("parameters",a(ps))+","+field("body",a(m.getBody()==null?Collections.emptyList():block(m.getBody())))+"}"; }
  public static void main(String[] args) throws Exception {
    if(args.length!=1) { System.err.println("usage: VzJavaFrontend <source>"); System.exit(64); }
    JavaCompiler c=ToolProvider.getSystemJavaCompiler(); if(c==null) { System.err.println("Java Compiler API unavailable"); System.exit(69); }
    DiagnosticCollector<JavaFileObject> ds=new DiagnosticCollector<>();
    StandardJavaFileManager fm=c.getStandardFileManager(ds,null,StandardCharsets.UTF_8);
    Iterable<? extends JavaFileObject> files=fm.getJavaFileObjects(new File(args[0]));
    JavacTask task=(JavacTask)c.getTask(null,fm,ds,Arrays.asList("-proc:none"),null,files);
    Iterable<? extends CompilationUnitTree> units; try { units=task.parse(); } catch(Exception e) { System.err.println(e.getMessage()); System.exit(2); return; }
    if(!ds.getDiagnostics().isEmpty()) { for(Diagnostic<?> d:ds.getDiagnostics()) if(d.getKind()==Diagnostic.Kind.ERROR) System.err.println(d.getMessage(Locale.ROOT)); if(ds.getDiagnostics().stream().anyMatch(d->d.getKind()==Diagnostic.Kind.ERROR)) { System.exit(2); return; } }
    List<String> methods=new ArrayList<>(); for(CompilationUnitTree u:units) for(Tree t:u.getTypeDecls()) if(t instanceof ClassTree) for(Tree m:((ClassTree)t).getMembers()) if(m instanceof MethodTree) methods.add(method((MethodTree)m));
    System.out.println("{\"methods\":"+a(methods)+"}");
  }
}

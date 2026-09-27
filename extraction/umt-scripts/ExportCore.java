import ghidra.app.script.GhidraScript;
import ghidra.app.decompiler.*;
import ghidra.program.model.listing.*;
import java.io.*;
import java.nio.file.*;
public class ExportCore extends GhidraScript {
 public void run() throws Exception {
  String b=getScriptArgs()[0];DecompInterface d=new DecompInterface();d.openProgram(currentProgram);
  try { for(String line:Files.readAllLines(Path.of(b,"core-functions.tsv"))) {
   String[] v=line.split("\t");Function f=getFunctionAt(toAddr(v[0]));
   if(f==null || !f.getBody().contains(toAddr(v[2]))) {println("REJECT "+line);continue;}
   DecompileResults r=d.decompileFunction(f,60,monitor);
   if(r.decompileCompleted()) {
    try(PrintWriter p=new PrintWriter(new File(b+"/exports-core/"+v[1]+".c"))) {
     p.println("/* Candidate mapping via RIP-relative LEA at "+v[2]+"; original function "+v[0]+". */");p.println(r.getDecompiledFunction().getC());
    }println("EXPORTED "+v[1]);
   }else println("FAILED "+v[1]+" "+r.getErrorMessage());
  }}finally{d.dispose();}
 }
}
